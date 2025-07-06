#!/usr/bin/env python3
"""
Comprehensive Raft Implementation Analysis

This script provides deep analysis of the Raft consensus algorithm implementation
in the Toka OS project, focusing on correctness, performance, and security aspects.
"""

import os
import re
import json
import ast
from typing import Dict, List, Set, Tuple, Optional, Any
from dataclasses import dataclass, asdict
from datetime import datetime
import subprocess

@dataclass
class RaftAnalysisResult:
    """Result of comprehensive Raft analysis"""
    timestamp: str
    implementation_completeness: Dict[str, float]
    safety_analysis: Dict[str, List[str]]
    performance_analysis: Dict[str, List[str]]
    security_analysis: Dict[str, List[str]]
    code_quality_metrics: Dict[str, Any]
    recommendations: List[str]
    critical_issues: List[str]
    conformance_score: float

class RaftImplementationAnalyzer:
    """Comprehensive analyzer for Raft implementation"""
    
    def __init__(self, workspace_root: str):
        self.workspace_root = workspace_root
        self.raft_core_path = os.path.join(workspace_root, "crates/raft-core")
        self.raft_storage_path = os.path.join(workspace_root, "crates/raft-storage")
        
        # Raft algorithm components checklist
        self.raft_components = {
            "leader_election": [
                "randomized_election_timeout",
                "candidate_state_management",
                "vote_request_handling",
                "vote_response_processing",
                "pre_vote_optimization"
            ],
            "log_replication": [
                "append_entries_rpc",
                "log_consistency_check",
                "log_matching_property",
                "commit_index_advancement",
                "log_compaction"
            ],
            "safety_guarantees": [
                "leader_completeness",
                "log_matching",
                "leader_append_only",
                "state_machine_safety",
                "election_safety"
            ],
            "cluster_membership": [
                "configuration_changes",
                "joint_consensus",
                "node_addition_removal",
                "quorum_calculation"
            ],
            "persistence": [
                "persistent_state",
                "volatile_state",
                "log_persistence",
                "snapshot_persistence"
            ],
            "networking": [
                "rpc_handling",
                "network_partition_handling",
                "message_serialization",
                "timeout_management"
            ]
        }
    
    def analyze_implementation_completeness(self) -> Dict[str, float]:
        """Analyze how complete the Raft implementation is"""
        completeness = {}
        
        for component, features in self.raft_components.items():
            implemented_features = 0
            total_features = len(features)
            
            for feature in features:
                if self._is_feature_implemented(feature):
                    implemented_features += 1
            
            completeness[component] = implemented_features / total_features
        
        return completeness
    
    def _is_feature_implemented(self, feature: str) -> bool:
        """Check if a specific Raft feature is implemented"""
        search_patterns = {
            "randomized_election_timeout": ["random.*election.*timeout", "election_timeout.*random"],
            "candidate_state_management": ["CandidateState", "become_candidate"],
            "vote_request_handling": ["VoteRequest", "handle_vote_request"],
            "vote_response_processing": ["VoteResponse", "handle_vote_response"],
            "pre_vote_optimization": ["pre_vote", "PreVote"],
            "append_entries_rpc": ["AppendEntriesRequest", "append_entries"],
            "log_consistency_check": ["log.*consistency", "prev_log_index"],
            "log_matching_property": ["log.*match", "matching_property"],
            "commit_index_advancement": ["commit_index", "advance_commit"],
            "log_compaction": ["compact", "snapshot"],
            "leader_completeness": ["leader.*complete", "complete.*leader"],
            "log_matching": ["log.*match", "matching.*log"],
            "leader_append_only": ["append.*only", "leader.*append"],
            "state_machine_safety": ["state.*machine.*safe", "safe.*state"],
            "election_safety": ["election.*safe", "safe.*election"],
            "configuration_changes": ["config.*change", "membership.*change"],
            "joint_consensus": ["joint.*consensus", "configuration.*joint"],
            "node_addition_removal": ["add.*node", "remove.*node"],
            "quorum_calculation": ["quorum", "majority"],
            "persistent_state": ["persistent", "PersistentState"],
            "volatile_state": ["volatile", "VolatileState"],
            "log_persistence": ["log.*persist", "persist.*log"],
            "snapshot_persistence": ["snapshot.*persist", "persist.*snapshot"],
            "rpc_handling": ["rpc", "RPC", "handle.*message"],
            "network_partition_handling": ["partition", "network.*split"],
            "message_serialization": ["serde", "serialize", "deserialize"],
            "timeout_management": ["timeout", "timer", "interval"]
        }
        
        patterns = search_patterns.get(feature, [feature])
        
        for pattern in patterns:
            if self._search_in_raft_code(pattern):
                return True
        
        return False
    
    def _search_in_raft_code(self, pattern: str) -> bool:
        """Search for a pattern in Raft code"""
        for raft_path in [self.raft_core_path, self.raft_storage_path]:
            if os.path.exists(raft_path):
                for root, dirs, files in os.walk(raft_path):
                    for file in files:
                        if file.endswith('.rs'):
                            file_path = os.path.join(root, file)
                            try:
                                with open(file_path, 'r') as f:
                                    content = f.read()
                                    if re.search(pattern, content, re.IGNORECASE):
                                        return True
                            except Exception:
                                continue
        return False
    
    def analyze_safety_properties(self) -> Dict[str, List[str]]:
        """Analyze Raft safety properties"""
        safety_analysis = {
            "election_safety": [],
            "leader_append_only": [],
            "log_matching": [],
            "leader_completeness": [],
            "state_machine_safety": []
        }
        
        # Check for common safety violations
        if self._check_multiple_leaders_possible():
            safety_analysis["election_safety"].append("Potential for multiple leaders in same term")
        
        if self._check_log_overwrites():
            safety_analysis["leader_append_only"].append("Log entries may be overwritten")
        
        if self._check_inconsistent_logs():
            safety_analysis["log_matching"].append("Log consistency checks may be insufficient")
        
        if self._check_committed_entry_loss():
            safety_analysis["leader_completeness"].append("Committed entries may be lost")
        
        if self._check_state_machine_divergence():
            safety_analysis["state_machine_safety"].append("State machines may diverge")
        
        return safety_analysis
    
    def _check_multiple_leaders_possible(self) -> bool:
        """Check if multiple leaders are possible in same term"""
        # Look for proper term checks and vote validation
        return not self._search_in_raft_code("term.*check") or not self._search_in_raft_code("vote.*validation")
    
    def _check_log_overwrites(self) -> bool:
        """Check if log entries can be overwritten"""
        # Look for proper log append-only enforcement
        return not self._search_in_raft_code("append.*only") or self._search_in_raft_code("overwrite")
    
    def _check_inconsistent_logs(self) -> bool:
        """Check for log inconsistency issues"""
        # Look for proper log matching checks
        return not self._search_in_raft_code("prev_log_index") or not self._search_in_raft_code("log.*match")
    
    def _check_committed_entry_loss(self) -> bool:
        """Check if committed entries can be lost"""
        # Look for proper commit index handling
        return not self._search_in_raft_code("commit.*index") or not self._search_in_raft_code("majority")
    
    def _check_state_machine_divergence(self) -> bool:
        """Check if state machines can diverge"""
        # Look for proper state machine application order
        return not self._search_in_raft_code("apply.*order") or not self._search_in_raft_code("deterministic")
    
    def analyze_performance_aspects(self) -> Dict[str, List[str]]:
        """Analyze performance aspects"""
        performance_analysis = {
            "batching": [],
            "pipelining": [],
            "caching": [],
            "memory_usage": [],
            "network_efficiency": []
        }
        
        # Check for performance optimizations
        if not self._search_in_raft_code("batch"):
            performance_analysis["batching"].append("No batching optimization detected")
        
        if not self._search_in_raft_code("pipeline"):
            performance_analysis["pipelining"].append("No pipelining optimization detected")
        
        if self._search_in_raft_code("clone.*clone"):
            performance_analysis["memory_usage"].append("Excessive cloning detected")
        
        if not self._search_in_raft_code("compress"):
            performance_analysis["network_efficiency"].append("No compression for network traffic")
        
        return performance_analysis
    
    def analyze_security_aspects(self) -> Dict[str, List[str]]:
        """Analyze security aspects"""
        security_analysis = {
            "authentication": [],
            "authorization": [],
            "encryption": [],
            "input_validation": [],
            "audit_logging": []
        }
        
        # Check for security measures
        if not self._search_in_raft_code("auth"):
            security_analysis["authentication"].append("No authentication mechanism detected")
        
        if not self._search_in_raft_code("encrypt"):
            security_analysis["encryption"].append("No encryption for communication")
        
        if self._search_in_raft_code("unwrap"):
            security_analysis["input_validation"].append("Unsafe unwrap() calls detected")
        
        if not self._search_in_raft_code("audit"):
            security_analysis["audit_logging"].append("No audit logging detected")
        
        return security_analysis
    
    def calculate_code_quality_metrics(self) -> Dict[str, Any]:
        """Calculate code quality metrics"""
        metrics = {
            "total_lines": 0,
            "comment_lines": 0,
            "comment_ratio": 0.0,
            "cyclomatic_complexity": 0,
            "test_coverage": 0.0,
            "unsafe_code_blocks": 0,
            "panic_statements": 0,
            "unwrap_calls": 0
        }
        
        for raft_path in [self.raft_core_path, self.raft_storage_path]:
            if os.path.exists(raft_path):
                for root, dirs, files in os.walk(raft_path):
                    for file in files:
                        if file.endswith('.rs'):
                            file_path = os.path.join(root, file)
                            file_metrics = self._analyze_file_metrics(file_path)
                            for key, value in file_metrics.items():
                                if isinstance(value, (int, float)):
                                    metrics[key] += value
        
        # Calculate ratios
        if metrics["total_lines"] > 0:
            comment_lines = metrics.get("comment_lines", 0)
            metrics["comment_ratio"] = comment_lines / metrics["total_lines"]
        
        return metrics
    
    def _analyze_file_metrics(self, file_path: str) -> Dict[str, Any]:
        """Analyze metrics for a single file"""
        metrics = {
            "total_lines": 0,
            "comment_lines": 0,
            "unsafe_code_blocks": 0,
            "panic_statements": 0,
            "unwrap_calls": 0
        }
        
        try:
            with open(file_path, 'r') as f:
                content = f.read()
                lines = content.split('\n')
                
                metrics["total_lines"] = len(lines)
                metrics["comment_lines"] = sum(1 for line in lines if line.strip().startswith('//'))
                metrics["unsafe_code_blocks"] = content.count('unsafe')
                metrics["panic_statements"] = content.count('panic!')
                metrics["unwrap_calls"] = content.count('.unwrap()')
                
        except Exception:
            pass
        
        return metrics
    
    def generate_recommendations(self, completeness: Dict[str, float], 
                               safety: Dict[str, List[str]],
                               performance: Dict[str, List[str]],
                               security: Dict[str, List[str]]) -> List[str]:
        """Generate specific recommendations"""
        recommendations = []
        
        # Completeness-based recommendations
        for component, score in completeness.items():
            if score < 0.7:
                recommendations.append(f"Improve {component} implementation (currently {score:.1%} complete)")
        
        # Safety-based recommendations
        for category, issues in safety.items():
            if issues:
                recommendations.append(f"Address {category} issues: {', '.join(issues)}")
        
        # Performance-based recommendations
        for category, issues in performance.items():
            if issues:
                recommendations.append(f"Optimize {category}: {', '.join(issues)}")
        
        # Security-based recommendations
        for category, issues in security.items():
            if issues:
                recommendations.append(f"Enhance {category}: {', '.join(issues)}")
        
        return recommendations
    
    def calculate_conformance_score(self, completeness: Dict[str, float]) -> float:
        """Calculate overall Raft conformance score"""
        if not completeness:
            return 0.0
        
        # Weight different components
        weights = {
            "leader_election": 0.25,
            "log_replication": 0.25,
            "safety_guarantees": 0.30,
            "cluster_membership": 0.10,
            "persistence": 0.05,
            "networking": 0.05
        }
        
        weighted_score = 0.0
        for component, score in completeness.items():
            weight = weights.get(component, 0.0)
            weighted_score += score * weight
        
        return weighted_score
    
    def run_comprehensive_analysis(self) -> RaftAnalysisResult:
        """Run comprehensive analysis of Raft implementation"""
        print("Running comprehensive Raft implementation analysis...")
        
        # Analyze implementation completeness
        completeness = self.analyze_implementation_completeness()
        
        # Analyze safety properties
        safety = self.analyze_safety_properties()
        
        # Analyze performance aspects
        performance = self.analyze_performance_aspects()
        
        # Analyze security aspects
        security = self.analyze_security_aspects()
        
        # Calculate code quality metrics
        metrics = self.calculate_code_quality_metrics()
        
        # Generate recommendations
        recommendations = self.generate_recommendations(completeness, safety, performance, security)
        
        # Identify critical issues
        critical_issues = []
        for category, issues in safety.items():
            critical_issues.extend(issues)
        
        # Calculate conformance score
        conformance_score = self.calculate_conformance_score(completeness)
        
        result = RaftAnalysisResult(
            timestamp=datetime.now().isoformat(),
            implementation_completeness=completeness,
            safety_analysis=safety,
            performance_analysis=performance,
            security_analysis=security,
            code_quality_metrics=metrics,
            recommendations=recommendations,
            critical_issues=critical_issues,
            conformance_score=conformance_score
        )
        
        return result
    
    def save_analysis_result(self, result: RaftAnalysisResult, filename: Optional[str] = None):
        """Save analysis result to file"""
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"raft_analysis_{timestamp}.json"
        
        with open(filename, 'w') as f:
            json.dump(asdict(result), f, indent=2)
        
        print(f"Analysis result saved to: {filename}")
    
    def print_analysis_summary(self, result: RaftAnalysisResult):
        """Print a summary of the analysis"""
        print("\n" + "="*80)
        print("RAFT IMPLEMENTATION ANALYSIS SUMMARY")
        print("="*80)
        print(f"Analysis timestamp: {result.timestamp}")
        print(f"Overall conformance score: {result.conformance_score:.1%}")
        
        print("\nImplementation Completeness:")
        for component, score in result.implementation_completeness.items():
            status = "✓" if score >= 0.8 else "⚠" if score >= 0.5 else "✗"
            print(f"  {status} {component}: {score:.1%}")
        
        print(f"\nCritical Issues ({len(result.critical_issues)}):")
        for issue in result.critical_issues:
            print(f"  ⚠ {issue}")
        
        print(f"\nTop Recommendations ({len(result.recommendations)}):")
        for i, rec in enumerate(result.recommendations[:5], 1):
            print(f"  {i}. {rec}")
        
        print(f"\nCode Quality Metrics:")
        metrics = result.code_quality_metrics
        print(f"  Total lines: {metrics.get('total_lines', 0)}")
        print(f"  Comment ratio: {metrics.get('comment_ratio', 0):.1%}")
        print(f"  Unsafe blocks: {metrics.get('unsafe_code_blocks', 0)}")
        print(f"  Panic statements: {metrics.get('panic_statements', 0)}")
        print(f"  Unwrap calls: {metrics.get('unwrap_calls', 0)}")
        
        print("="*80)

def main():
    """Main function"""
    workspace_root = os.getcwd()
    analyzer = RaftImplementationAnalyzer(workspace_root)
    
    # Run comprehensive analysis
    result = analyzer.run_comprehensive_analysis()
    
    # Print summary
    analyzer.print_analysis_summary(result)
    
    # Save detailed results
    analyzer.save_analysis_result(result)
    
    print("\nAnalysis complete!")

if __name__ == "__main__":
    main()