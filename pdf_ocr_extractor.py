#!/usr/bin/env python3
"""
PDF OCR Text Extractor

This script uses OCR to extract text from all PDF files in the current directory
and its subdirectories. It handles both text-based PDFs and image-based PDFs.

Requirements:
    pip install PyPDF2 pytesseract pdf2image pillow

System requirements:
    - tesseract-ocr (sudo apt-get install tesseract-ocr on Ubuntu/Debian)
    - poppler-utils (sudo apt-get install poppler-utils on Ubuntu/Debian)
"""

import os
import sys
from pathlib import Path
import PyPDF2
import pytesseract
from pdf2image import convert_from_path
from PIL import Image
import logging
from concurrent.futures import ThreadPoolExecutor, as_completed
from threading import Lock
import argparse
import sqlite3
from datetime import datetime
import hashlib

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# Thread-safe database writing
db_lock = Lock()

def init_database(db_path):
    """Initialize SQLite database with schema"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS pdf_extractions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL,
            file_hash TEXT,
            file_size INTEGER,
            extraction_method TEXT NOT NULL,
            extracted_text TEXT,
            page_count INTEGER,
            processing_time_seconds REAL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            error_message TEXT,
            UNIQUE(file_path, file_hash)
        )
    """)
    
    cursor.execute("""
        CREATE INDEX IF NOT EXISTS idx_file_path ON pdf_extractions(file_path)
    """)
    
    cursor.execute("""
        CREATE INDEX IF NOT EXISTS idx_extraction_method ON pdf_extractions(extraction_method)
    """)
    
    cursor.execute("""
        CREATE INDEX IF NOT EXISTS idx_timestamp ON pdf_extractions(timestamp)
    """)
    
    conn.commit()
    conn.close()
    logger.info(f"Database initialized: {db_path}")

def get_file_hash(file_path):
    """Get SHA256 hash of file for deduplication"""
    try:
        with open(file_path, 'rb') as f:
            file_hash = hashlib.sha256()
            while chunk := f.read(8192):
                file_hash.update(chunk)
            return file_hash.hexdigest()
    except Exception as e:
        logger.warning(f"Could not hash file {file_path}: {e}")
        return None

def extract_text_from_pdf(pdf_path):
    """
    Extract text from PDF using PyPDF2 first, then OCR if needed
    
    Args:
        pdf_path (str): Path to the PDF file
        
    Returns:
        dict: {
            'text': str,
            'method': str ('direct' or 'ocr'),
            'page_count': int,
            'success': bool,
            'error': str or None
        }
    """
    result = {
        'text': "",
        'method': 'direct',
        'page_count': 0,
        'success': False,
        'error': None
    }
    
    try:
        # First try to extract text directly from PDF
        with open(pdf_path, 'rb') as file:
            pdf_reader = PyPDF2.PdfReader(file)
            result['page_count'] = len(pdf_reader.pages)
            
            for page_num, page in enumerate(pdf_reader.pages):
                page_text = page.extract_text()
                if page_text.strip():
                    result['text'] += f"--- Page {page_num + 1} ---\n"
                    result['text'] += page_text + "\n\n"
        
        # If we got substantial text, return it
        if len(result['text'].strip()) > 50:
            logger.info(f"Extracted text directly from {pdf_path}")
            result['success'] = True
            return result
            
    except Exception as e:
        logger.warning(f"Direct text extraction failed for {pdf_path}: {e}")
        result['error'] = f"Direct extraction failed: {e}"
    
    # If direct extraction failed or yielded little text, use OCR
    try:
        logger.info(f"Using OCR for {pdf_path}")
        result['method'] = 'ocr'
        images = convert_from_path(pdf_path)
        result['page_count'] = len(images)
        ocr_text = ""
        
        for page_num, image in enumerate(images):
            page_text = pytesseract.image_to_string(image, lang='eng')
            if page_text.strip():
                ocr_text += f"--- Page {page_num + 1} (OCR) ---\n"
                ocr_text += page_text + "\n\n"
        
        result['text'] = ocr_text
        result['success'] = True
        result['error'] = None
        return result
        
    except Exception as e:
        logger.error(f"OCR extraction failed for {pdf_path}: {e}")
        result['text'] = ""
        result['success'] = False
        result['error'] = f"OCR extraction failed: {e}"
        return result

def find_all_pdfs(directory):
    """
    Find all PDF files in directory and subdirectories
    
    Args:
        directory (str): Root directory to search
        
    Returns:
        list: List of PDF file paths
    """
    pdf_files = []
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.lower().endswith('.pdf'):
                pdf_files.append(os.path.join(root, file))
    
    return sorted(pdf_files)

def process_single_pdf(pdf_path, current_dir, pdf_index, total_pdfs, db_path):
    """Process a single PDF file and store results in database"""
    import time
    
    relative_path = os.path.relpath(pdf_path, current_dir)
    logger.info(f"Processing ({pdf_index}/{total_pdfs}): {relative_path}")
    
    start_time = time.time()
    
    try:
        # Get file metadata
        file_size = os.path.getsize(pdf_path)
        file_hash = get_file_hash(pdf_path)
        
        # Extract text
        extraction_result = extract_text_from_pdf(pdf_path)
        processing_time = time.time() - start_time
        
        # Store in database
        store_extraction_result(
            db_path=db_path,
            file_path=relative_path,
            file_hash=file_hash,
            file_size=file_size,
            extraction_method=extraction_result['method'],
            extracted_text=extraction_result['text'],
            page_count=extraction_result['page_count'],
            processing_time=processing_time,
            success=extraction_result['success'],
            error_message=extraction_result['error']
        )
        
        return {
            'path': relative_path,
            'success': extraction_result['success'],
            'method': extraction_result['method'],
            'processing_time': processing_time
        }
        
    except Exception as e:
        processing_time = time.time() - start_time
        error_msg = f"ERROR processing {relative_path}: {e}"
        logger.error(error_msg)
        
        # Store error in database
        store_extraction_result(
            db_path=db_path,
            file_path=relative_path,
            file_hash=None,
            file_size=0,
            extraction_method='error',
            extracted_text='',
            page_count=0,
            processing_time=processing_time,
            success=False,
            error_message=str(e)
        )
        
        return {
            'path': relative_path,
            'success': False,
            'method': 'error',
            'processing_time': processing_time,
            'error': str(e)
        }

def store_extraction_result(db_path, file_path, file_hash, file_size, extraction_method, 
                          extracted_text, page_count, processing_time, success, error_message):
    """Thread-safe storage of extraction results in database"""
    with db_lock:
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        
        try:
            cursor.execute("""
                INSERT OR REPLACE INTO pdf_extractions 
                (file_path, file_hash, file_size, extraction_method, extracted_text, 
                 page_count, processing_time_seconds, success, error_message)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (file_path, file_hash, file_size, extraction_method, extracted_text,
                  page_count, processing_time, success, error_message))
            
            conn.commit()
            
        except sqlite3.Error as e:
            logger.error(f"Database error storing {file_path}: {e}")
        finally:
            conn.close()

def main():
    """Main function to process all PDFs with multithreading and SQLite storage"""
    parser = argparse.ArgumentParser(description='Extract text from PDF files using OCR')
    parser.add_argument('--threads', '-t', type=int, default=4, 
                       help='Number of worker threads (default: 4)')
    parser.add_argument('--database', '-d', type=str, default='pdf_extractions.db',
                       help='SQLite database file (default: pdf_extractions.db)')
    parser.add_argument('--export-txt', type=str, 
                       help='Export results to text file after processing')
    
    args = parser.parse_args()
    
    current_dir = os.getcwd()
    db_path = os.path.join(current_dir, args.database)
    
    # Initialize database
    init_database(db_path)
    
    logger.info(f"Searching for PDF files in: {current_dir}")
    pdf_files = find_all_pdfs(current_dir)
    
    if not pdf_files:
        print("No PDF files found in the current directory or its subdirectories.")
        return
    
    logger.info(f"Found {len(pdf_files)} PDF files")
    logger.info(f"Using {args.threads} worker threads")
    logger.info(f"Database: {db_path}")
    
    # Process PDFs with multithreading
    completed = 0
    success_count = 0
    
    with ThreadPoolExecutor(max_workers=args.threads) as executor:
        # Submit all tasks
        future_to_pdf = {
            executor.submit(process_single_pdf, pdf_path, current_dir, i, len(pdf_files), db_path): pdf_path 
            for i, pdf_path in enumerate(pdf_files, 1)
        }
        
        # Process completed tasks as they finish
        for future in as_completed(future_to_pdf):
            pdf_path = future_to_pdf[future]
            try:
                result = future.result()
                completed += 1
                
                if result['success']:
                    success_count += 1
                    logger.info(f"Completed ({completed}/{len(pdf_files)}): {result['path']} "
                              f"[{result['method']}] ({result['processing_time']:.1f}s)")
                else:
                    logger.warning(f"Failed ({completed}/{len(pdf_files)}): {result['path']} "
                                 f"({result['processing_time']:.1f}s)")
                    
            except Exception as e:
                logger.error(f"Unexpected error processing {pdf_path}: {e}")
                completed += 1
    
    # Print summary
    print(f"\nExtraction complete!")
    print(f"Processed: {completed}/{len(pdf_files)} PDFs")
    print(f"Successful: {success_count}")
    print(f"Failed: {completed - success_count}")
    print(f"Database: {db_path}")
    
    # Export to text file if requested
    if args.export_txt:
        export_to_text_file(db_path, args.export_txt)
        print(f"Results exported to: {args.export_txt}")

def export_to_text_file(db_path, output_file):
    """Export database results to text file"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    cursor.execute("""
        SELECT file_path, extraction_method, extracted_text, success, error_message, timestamp
        FROM pdf_extractions 
        ORDER BY file_path
    """)
    
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write("PDF Text Extraction Results\n")
        f.write("=" * 80 + "\n\n")
        
        for row in cursor.fetchall():
            file_path, method, text, success, error, timestamp = row
            
            f.write(f"FILE: {file_path}\n")
            f.write(f"METHOD: {method}\n")
            f.write(f"SUCCESS: {success}\n")
            f.write(f"TIMESTAMP: {timestamp}\n")
            f.write("=" * 80 + "\n")
            
            if success and text:
                f.write(text)
            elif error:
                f.write(f"ERROR: {error}\n")
            
            f.write("\n" + "=" * 80 + "\n\n")
    
    conn.close()

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nOperation cancelled by user")
        sys.exit(1)
    except Exception as e:
        logger.error(f"Unexpected error: {e}")
        sys.exit(1)