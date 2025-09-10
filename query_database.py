#!/usr/bin/env python3
"""
Query and analyze PDF extraction database

This script provides various ways to query and analyze the SQLite database
containing PDF text extraction results.
"""

import sqlite3
import argparse
import json
from datetime import datetime

def connect_db(db_path):
    """Connect to SQLite database"""
    try:
        conn = sqlite3.connect(db_path)
        conn.row_factory = sqlite3.Row  # Enable column access by name
        return conn
    except sqlite3.Error as e:
        print(f"Database connection error: {e}")
        return None

def show_stats(db_path):
    """Show extraction statistics"""
    conn = connect_db(db_path)
    if not conn:
        return
    
    cursor = conn.cursor()
    
    # Overall stats
    cursor.execute("SELECT COUNT(*) as total FROM pdf_extractions")
    total = cursor.fetchone()['total']
    
    cursor.execute("SELECT COUNT(*) as success FROM pdf_extractions WHERE success = 1")
    success = cursor.fetchone()['success']
    
    cursor.execute("SELECT COUNT(*) as failed FROM pdf_extractions WHERE success = 0")
    failed = cursor.fetchone()['failed']
    
    # Method breakdown
    cursor.execute("""
        SELECT extraction_method, COUNT(*) as count 
        FROM pdf_extractions 
        GROUP BY extraction_method
    """)
    methods = cursor.fetchall()
    
    # Average processing time
    cursor.execute("SELECT AVG(processing_time_seconds) as avg_time FROM pdf_extractions WHERE success = 1")
    avg_time = cursor.fetchone()['avg_time'] or 0
    
    # File size stats
    cursor.execute("SELECT AVG(file_size), MIN(file_size), MAX(file_size) FROM pdf_extractions WHERE success = 1")
    size_stats = cursor.fetchone()
    
    print("PDF Extraction Database Statistics")
    print("=" * 40)
    print(f"Total files processed: {total}")
    print(f"Successful extractions: {success}")
    print(f"Failed extractions: {failed}")
    print(f"Success rate: {(success/total*100):.1f}%" if total > 0 else "N/A")
    print(f"Average processing time: {avg_time:.2f}s")
    
    if size_stats[0]:
        print(f"Average file size: {size_stats[0]/1024:.1f} KB")
        print(f"File size range: {size_stats[1]/1024:.1f} - {size_stats[2]/1024:.1f} KB")
    
    print("\nExtraction methods:")
    for method in methods:
        print(f"  {method['extraction_method']}: {method['count']} files")
    
    conn.close()

def search_text(db_path, query, limit=10):
    """Search for text in extracted content"""
    conn = connect_db(db_path)
    if not conn:
        return
    
    cursor = conn.cursor()
    cursor.execute("""
        SELECT file_path, extraction_method, 
               substr(extracted_text, 1, 200) as preview,
               timestamp
        FROM pdf_extractions 
        WHERE extracted_text LIKE ? AND success = 1
        ORDER BY timestamp DESC
        LIMIT ?
    """, (f'%{query}%', limit))
    
    results = cursor.fetchall()
    
    print(f"Search results for '{query}' (showing {len(results)} of max {limit}):")
    print("=" * 60)
    
    for result in results:
        print(f"File: {result['file_path']}")
        print(f"Method: {result['extraction_method']}")
        print(f"Timestamp: {result['timestamp']}")
        print(f"Preview: {result['preview']}...")
        print("-" * 40)
    
    conn.close()

def list_files(db_path, method=None, success_only=True):
    """List processed files with optional filtering"""
    conn = connect_db(db_path)
    if not conn:
        return
    
    cursor = conn.cursor()
    
    query = "SELECT file_path, extraction_method, success, processing_time_seconds, timestamp FROM pdf_extractions"
    params = []
    
    conditions = []
    if method:
        conditions.append("extraction_method = ?")
        params.append(method)
    
    if success_only:
        conditions.append("success = 1")
    
    if conditions:
        query += " WHERE " + " AND ".join(conditions)
    
    query += " ORDER BY timestamp DESC"
    
    cursor.execute(query, params)
    results = cursor.fetchall()
    
    print(f"Files in database ({len(results)} results):")
    print("=" * 80)
    
    for result in results:
        status = "✓" if result['success'] else "✗"
        print(f"{status} {result['file_path']} [{result['extraction_method']}] "
              f"({result['processing_time_seconds']:.1f}s) - {result['timestamp']}")
    
    conn.close()

def export_json(db_path, output_file, include_text=False):
    """Export database to JSON"""
    conn = connect_db(db_path)
    if not conn:
        return
    
    cursor = conn.cursor()
    
    if include_text:
        cursor.execute("SELECT * FROM pdf_extractions ORDER BY file_path")
    else:
        cursor.execute("""
            SELECT id, file_path, file_hash, file_size, extraction_method, 
                   page_count, processing_time_seconds, timestamp, success, error_message
            FROM pdf_extractions ORDER BY file_path
        """)
    
    results = []
    for row in cursor.fetchall():
        results.append(dict(row))
    
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(results, f, indent=2, default=str)
    
    print(f"Exported {len(results)} records to {output_file}")
    conn.close()

def main():
    parser = argparse.ArgumentParser(description='Query PDF extraction database')
    parser.add_argument('--database', '-d', default='pdf_extractions.db',
                       help='SQLite database file')
    
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Stats command
    subparsers.add_parser('stats', help='Show extraction statistics')
    
    # Search command
    search_parser = subparsers.add_parser('search', help='Search extracted text')
    search_parser.add_argument('query', help='Text to search for')
    search_parser.add_argument('--limit', type=int, default=10, help='Max results')
    
    # List command
    list_parser = subparsers.add_parser('list', help='List processed files')
    list_parser.add_argument('--method', choices=['direct', 'ocr', 'error'], 
                           help='Filter by extraction method')
    list_parser.add_argument('--all', action='store_true', 
                           help='Include failed extractions')
    
    # Export command
    export_parser = subparsers.add_parser('export', help='Export to JSON')
    export_parser.add_argument('output', help='Output JSON file')
    export_parser.add_argument('--include-text', action='store_true',
                             help='Include extracted text in export')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    if args.command == 'stats':
        show_stats(args.database)
    elif args.command == 'search':
        search_text(args.database, args.query, args.limit)
    elif args.command == 'list':
        list_files(args.database, args.method, not args.all)
    elif args.command == 'export':
        export_json(args.database, args.output, args.include_text)

if __name__ == "__main__":
    main()