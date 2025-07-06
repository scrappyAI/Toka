#!/usr/bin/env python3
"""
Raft Development Monitor for Toka OS

This script monitors changes to raft-related files and provides improvement
suggestions based on Raft consensus algorithm best practices.
"""

import os
import subprocess
import json
import time
from datetime import datetime
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, asdict
import threading
import hashlib

@dataclass
class RaftComponent:
    """Represents a raft-related component being monitored"""
    name: str
    path: str
    last_modified: float
    file_hash: str
    issues: List[str]
    suggestions: List[str]

@dataclass
class MonitoringReport:
    """Report of monitoring findings"""
    timestamp: str
    components: List[RaftComponent]
    recent_changes: List[str]
    improvement_suggestions: List[str]
    security_recommendations: List[str]
    performance_recommendations: List[str]

class RaftDevelopmentMonitor:
    """Monitor for Raft consensus algorithm development"""
    
    def __init__(self, workspace_root: str):
        self.workspace_root = workspace_root
        self.raft_paths = [
            "crates/raft-core/",
            "crates/raft-storage/",
        ]
        self.monitoring_active = False
        self.last_check = time.time()
        self.components = {}
        self.reports_dir = "raft_monitoring_reports"
        
        # Create reports directory
        os.makedirs(self.reports_dir, exist_ok=True)
        
        # Initialize component tracking
        self._initialize_components()
    
    def _initialize_components(self):
        """Initialize tracking for all raft components"""
        for raft_path in self.raft_paths:
            full_path = os.path.join(self.workspace_root, raft_path)
            if os.path.exists(full_path):
                self._scan_directory(full_path, raft_path)
    
    def _scan_directory(self, directory: str, relative_path: str):
        """Scan directory for raft files"""
        for root, dirs, files in os.walk(directory):
            # Skip target directories
            if 'target' in dirs:
                dirs.remove('target')
            
            for file in files:
                if file.endswith('.rs') or file.endswith('.toml'):
                    file_path = os.path.join(root, file)
                    rel_path = os.path.relpath(file_path, self.workspace_root)
                    
                    component = RaftComponent(
                        name=file,
                        path=rel_path,
                        last_modified=os.path.getmtime(file_path),
                        file_hash=self._calculate_file_hash(file_path),
                        issues=[],
                        suggestions=[]
                    )
                    
                    self.components[rel_path] = component
    
    def _calculate_file_hash(self, file_path: str) -> str:
        """Calculate hash of file content"""
        try:
            with open(file_path, 'rb') as f:
                return hashlib.sha256(f.read()).hexdigest()
        except Exception:
            return ""
    
    def _get_recent_commits(self, since_hours: int = 24) -> List[str]:
        """Get recent git commits related to raft"""
        try:
            cmd = [
                'git', 'log', '--oneline', '--grep=raft', '-i',
                f'--since={since_hours} hours ago', '--all'
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=self.workspace_root)
            return result.stdout.strip().split('\n') if result.stdout.strip() else []
        except Exception:
            return []
    
    def _analyze_raft_code(self, file_path: str) -> tuple[List[str], List[str]]:
        """Analyze raft code for issues and suggestions"""
        issues = []
        suggestions = []
        
        try:
            with open(file_path, 'r') as f:
                content = f.read()
            
            # Security analysis
            if 'unsafe' in content:
                issues.append("Contains unsafe code - review for memory safety")
            
            if 'unwrap()' in content:
                issues.append("Contains unwrap() calls - consider proper error handling")
            
            if 'panic!' in content:
                issues.append("Contains panic! calls - may cause instability")
            
            # Raft-specific analysis
            if 'leader_election' in content.lower():
                suggestions.append("Consider implementing pre-vote optimization for leader election")
            
            if 'append_entries' in content.lower():
                suggestions.append("Ensure batch optimization for AppendEntries requests")
            
            if 'heartbeat' in content.lower():
                suggestions.append("Consider adaptive heartbeat intervals based on network conditions")
            
            if 'snapshot' in content.lower():
                suggestions.append("Implement incremental snapshots for better performance")
            
            if 'log_replication' in content.lower():
                suggestions.append("Consider pipeline optimization for log replication")
            
            # Performance analysis
            if '.clone()' in content and content.count('.clone()') > 5:
                suggestions.append("High clone usage detected - consider using references")
            
            if 'HashMap' in content:
                suggestions.append("Consider using BTreeMap for deterministic iteration order")
            
            if 'std::thread::sleep' in content:
                suggestions.append("Consider using tokio::time::sleep for async compatibility")
            
            # Documentation analysis
            if content.count('///') < content.count('pub fn'):
                suggestions.append("Add more comprehensive documentation for public APIs")
            
        except Exception as e:
            issues.append(f"Error analyzing file: {e}")
        
        return issues, suggestions
    
    def _generate_improvement_suggestions(self) -> List[str]:
        """Generate general improvement suggestions for raft implementation"""
        return [
            "Implement Byzantine fault tolerance extensions for enhanced security",
            "Add metrics collection for leader election frequency and log replication latency",
            "Implement dynamic cluster membership changes (joint consensus)",
            "Add configuration validation for election timeout ranges",
            "Implement log compaction with configurable retention policies",
            "Add chaos engineering tests for network partition scenarios",
            "Implement leader stickiness to reduce unnecessary elections",
            "Add detailed tracing for debugging consensus issues",
            "Implement automatic cluster health monitoring",
            "Add support for read-only replicas",
            "Implement efficient log shipping for catching up slow followers",
            "Add automated backup and recovery mechanisms"
        ]
    
    def _generate_security_recommendations(self) -> List[str]:
        """Generate security-focused recommendations"""
        return [
            "Implement message authentication to prevent byzantine attacks",
            "Add rate limiting for RPC requests to prevent DoS attacks",
            "Implement secure random number generation for election timeouts",
            "Add audit logging for all consensus operations",
            "Implement proper input validation for all RPC messages",
            "Add encryption for inter-node communication",
            "Implement capability-based security for admin operations",
            "Add detection for split-brain scenarios",
            "Implement proper cleanup of sensitive data in memory",
            "Add protection against time-based attacks on election timeouts"
        ]
    
    def _generate_performance_recommendations(self) -> List[str]:
        """Generate performance-focused recommendations"""
        return [
            "Implement batching for log entries to reduce I/O overhead",
            "Add memory pool for frequently allocated objects",
            "Implement zero-copy message serialization where possible",
            "Add adaptive timeouts based on network conditions",
            "Implement efficient log storage with write-ahead logging",
            "Add compression for large log entries",
            "Implement parallel processing for independent operations",
            "Add connection pooling for inter-node communication",
            "Implement efficient snapshot transfer mechanisms",
            "Add caching for frequently accessed configuration data"
        ]
    
    def check_for_changes(self) -> bool:
        """Check if any raft files have changed"""
        changes_detected = False
        
        for path, component in self.components.items():
            full_path = os.path.join(self.workspace_root, path)
            if os.path.exists(full_path):
                current_hash = self._calculate_file_hash(full_path)
                current_modified = os.path.getmtime(full_path)
                
                if (current_hash != component.file_hash or 
                    current_modified > component.last_modified):
                    changes_detected = True
                    component.file_hash = current_hash
                    component.last_modified = current_modified
                    
                    # Analyze the changed file
                    issues, suggestions = self._analyze_raft_code(full_path)
                    component.issues = issues
                    component.suggestions = suggestions
        
        return changes_detected
    
    def generate_report(self) -> MonitoringReport:
        """Generate a comprehensive monitoring report"""
        report = MonitoringReport(
            timestamp=datetime.now().isoformat(),
            components=list(self.components.values()),
            recent_changes=self._get_recent_commits(),
            improvement_suggestions=self._generate_improvement_suggestions(),
            security_recommendations=self._generate_security_recommendations(),
            performance_recommendations=self._generate_performance_recommendations()
        )
        
        return report
    
    def save_report(self, report: MonitoringReport):
        """Save monitoring report to file"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"raft_monitoring_report_{timestamp}.json"
        filepath = os.path.join(self.reports_dir, filename)
        
        with open(filepath, 'w') as f:
            json.dump(asdict(report), f, indent=2)
        
        print(f"Monitoring report saved to: {filepath}")
    
    def start_monitoring(self, interval_seconds: int = 60):
        """Start continuous monitoring"""
        self.monitoring_active = True
        print(f"Starting Raft development monitoring (interval: {interval_seconds}s)")
        
        def monitor_loop():
            while self.monitoring_active:
                try:
                    if self.check_for_changes():
                        print("Changes detected in raft code - generating report...")
                        report = self.generate_report()
                        self.save_report(report)
                        self._print_summary(report)
                    
                    time.sleep(interval_seconds)
                except Exception as e:
                    print(f"Error in monitoring loop: {e}")
                    time.sleep(interval_seconds)
        
        monitor_thread = threading.Thread(target=monitor_loop, daemon=True)
        monitor_thread.start()
        
        return monitor_thread
    
    def stop_monitoring(self):
        """Stop continuous monitoring"""
        self.monitoring_active = False
        print("Stopping Raft development monitoring")
    
    def _print_summary(self, report: MonitoringReport):
        """Print a summary of the monitoring report"""
        print("\n" + "="*80)
        print("RAFT DEVELOPMENT MONITORING REPORT")
        print("="*80)
        print(f"Timestamp: {report.timestamp}")
        print(f"Components monitored: {len(report.components)}")
        print(f"Recent commits: {len(report.recent_changes)}")
        
        # Print recent changes
        if report.recent_changes:
            print("\nRecent Changes:")
            for change in report.recent_changes[:5]:  # Show last 5
                print(f"  - {change}")
        
        # Print top issues
        all_issues = []
        for component in report.components:
            all_issues.extend(component.issues)
        
        if all_issues:
            print(f"\nTop Issues Found ({len(all_issues)} total):")
            unique_issues = list(set(all_issues))[:5]  # Show top 5 unique issues
            for issue in unique_issues:
                print(f"  - {issue}")
        
        # Print top suggestions
        all_suggestions = []
        for component in report.components:
            all_suggestions.extend(component.suggestions)
        
        if all_suggestions:
            print(f"\nTop Suggestions ({len(all_suggestions)} total):")
            unique_suggestions = list(set(all_suggestions))[:5]  # Show top 5 unique suggestions
            for suggestion in unique_suggestions:
                print(f"  - {suggestion}")
        
        print("\nFull report saved to:", self.reports_dir)
        print("="*80)

def main():
    """Main function to run the monitoring system"""
    workspace_root = os.getcwd()
    monitor = RaftDevelopmentMonitor(workspace_root)
    
    # Generate initial report
    print("Generating initial Raft development report...")
    initial_report = monitor.generate_report()
    monitor.save_report(initial_report)
    monitor._print_summary(initial_report)
    
    # Start continuous monitoring
    monitor_thread = monitor.start_monitoring(interval_seconds=30)
    
    try:
        print("Monitoring active. Press Ctrl+C to stop.")
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("\nReceived interrupt signal")
        monitor.stop_monitoring()
        monitor_thread.join(timeout=5)
        print("Monitoring stopped.")

if __name__ == "__main__":
    main()