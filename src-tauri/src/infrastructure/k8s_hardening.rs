//! Kubernetes Hardening Auditor
//!
//! Security checks based on NSA/CISA Kubernetes Hardening Guide
//! and CIS Kubernetes Benchmark.

use crate::infrastructure::models::*;
use uuid::Uuid;

/// Get all K8s hardening checks
pub fn get_k8s_hardening_checks() -> Vec<K8sHardeningCheck> {
    vec![
        // Pod Security
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::PodSecurity,
            title: "Non-root containers".to_string(),
            description: "Containers should run as non-root user".to_string(),
            rationale: "Running as root increases the attack surface and potential impact of container escape vulnerabilities".to_string(),
            remediation: "Set securityContext.runAsNonRoot: true and specify runAsUser in pod/container spec".to_string(),
            severity: Severity::High,
            cis_benchmark: Some("5.2.6".to_string()),
            nsa_reference: Some("Pod Security".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::PodSecurity,
            title: "Read-only root filesystem".to_string(),
            description: "Container root filesystem should be read-only".to_string(),
            rationale: "Prevents attackers from writing malicious files to the container filesystem".to_string(),
            remediation: "Set securityContext.readOnlyRootFilesystem: true".to_string(),
            severity: Severity::Medium,
            cis_benchmark: Some("5.2.4".to_string()),
            nsa_reference: Some("Pod Security".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::PodSecurity,
            title: "Privilege escalation disabled".to_string(),
            description: "Containers should not allow privilege escalation".to_string(),
            rationale: "Prevents processes from gaining more privileges than their parent".to_string(),
            remediation: "Set securityContext.allowPrivilegeEscalation: false".to_string(),
            severity: Severity::High,
            cis_benchmark: Some("5.2.5".to_string()),
            nsa_reference: Some("Pod Security".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::PodSecurity,
            title: "No privileged containers".to_string(),
            description: "Containers should not run in privileged mode".to_string(),
            rationale: "Privileged containers have full access to host resources".to_string(),
            remediation: "Set securityContext.privileged: false or remove the setting".to_string(),
            severity: Severity::Critical,
            cis_benchmark: Some("5.2.1".to_string()),
            nsa_reference: Some("Pod Security".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::PodSecurity,
            title: "Capabilities dropped".to_string(),
            description: "All unnecessary Linux capabilities should be dropped".to_string(),
            rationale: "Reduces the kernel attack surface available to containers".to_string(),
            remediation: "Set securityContext.capabilities.drop: ['ALL'] and only add required capabilities".to_string(),
            severity: Severity::Medium,
            cis_benchmark: Some("5.2.7".to_string()),
            nsa_reference: Some("Pod Security".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::PodSecurity,
            title: "Host namespaces not shared".to_string(),
            description: "Pods should not share host PID, IPC, or network namespaces".to_string(),
            rationale: "Sharing host namespaces breaks container isolation".to_string(),
            remediation: "Set hostPID: false, hostIPC: false, hostNetwork: false".to_string(),
            severity: Severity::High,
            cis_benchmark: Some("5.2.2".to_string()),
            nsa_reference: Some("Pod Security".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::PodSecurity,
            title: "Resource limits defined".to_string(),
            description: "CPU and memory limits should be set for all containers".to_string(),
            rationale: "Prevents resource exhaustion and denial of service".to_string(),
            remediation: "Set resources.limits.cpu and resources.limits.memory for all containers".to_string(),
            severity: Severity::Medium,
            cis_benchmark: Some("5.4.1".to_string()),
            nsa_reference: Some("Pod Security".to_string()),
        },

        // Network Policies
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::NetworkPolicies,
            title: "Default deny ingress policy".to_string(),
            description: "Default deny ingress network policy should exist in each namespace".to_string(),
            rationale: "Ensures pods can only receive traffic that is explicitly allowed".to_string(),
            remediation: "Create NetworkPolicy with empty podSelector and empty ingress rules".to_string(),
            severity: Severity::High,
            cis_benchmark: Some("5.3.2".to_string()),
            nsa_reference: Some("Network Separation".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::NetworkPolicies,
            title: "Default deny egress policy".to_string(),
            description: "Default deny egress network policy should exist in each namespace".to_string(),
            rationale: "Prevents unauthorized outbound connections from pods".to_string(),
            remediation: "Create NetworkPolicy with empty podSelector and policyTypes: ['Egress']".to_string(),
            severity: Severity::Medium,
            cis_benchmark: Some("5.3.2".to_string()),
            nsa_reference: Some("Network Separation".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::NetworkPolicies,
            title: "CNI supports network policies".to_string(),
            description: "Container Network Interface (CNI) should support NetworkPolicy enforcement".to_string(),
            rationale: "NetworkPolicy resources have no effect without supporting CNI".to_string(),
            remediation: "Deploy a CNI plugin that supports NetworkPolicy (Calico, Cilium, Weave)".to_string(),
            severity: Severity::High,
            cis_benchmark: None,
            nsa_reference: Some("Network Separation".to_string()),
        },

        // Authentication
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Authentication,
            title: "Anonymous authentication disabled".to_string(),
            description: "API server should have anonymous authentication disabled".to_string(),
            rationale: "Prevents unauthenticated access to the Kubernetes API".to_string(),
            remediation: "Set --anonymous-auth=false on API server".to_string(),
            severity: Severity::Critical,
            cis_benchmark: Some("1.2.1".to_string()),
            nsa_reference: Some("Authentication".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Authentication,
            title: "Strong authentication method".to_string(),
            description: "Use OIDC or client certificates for user authentication".to_string(),
            rationale: "Static tokens and basic auth are insecure".to_string(),
            remediation: "Configure OIDC provider or use X.509 client certificates".to_string(),
            severity: Severity::High,
            cis_benchmark: Some("3.1.1".to_string()),
            nsa_reference: Some("Authentication".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Authentication,
            title: "Service account token automount disabled".to_string(),
            description: "Disable automatic mounting of service account tokens when not needed".to_string(),
            rationale: "Reduces risk of token theft from compromised pods".to_string(),
            remediation: "Set automountServiceAccountToken: false on pods/service accounts that don't need API access".to_string(),
            severity: Severity::Medium,
            cis_benchmark: Some("5.1.6".to_string()),
            nsa_reference: Some("Authentication".to_string()),
        },

        // Authorization (RBAC)
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Authorization,
            title: "RBAC enabled".to_string(),
            description: "Role-Based Access Control should be enabled".to_string(),
            rationale: "RBAC provides fine-grained access control".to_string(),
            remediation: "Ensure --authorization-mode includes RBAC".to_string(),
            severity: Severity::Critical,
            cis_benchmark: Some("1.2.8".to_string()),
            nsa_reference: Some("Authorization".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Authorization,
            title: "No cluster-admin for users".to_string(),
            description: "cluster-admin role should not be bound to regular users".to_string(),
            rationale: "cluster-admin provides full access to the entire cluster".to_string(),
            remediation: "Use namespace-scoped roles with least privilege".to_string(),
            severity: Severity::High,
            cis_benchmark: Some("5.1.1".to_string()),
            nsa_reference: Some("Authorization".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Authorization,
            title: "Minimize wildcard permissions".to_string(),
            description: "Roles should not use wildcard (*) permissions".to_string(),
            rationale: "Wildcards grant excessive permissions".to_string(),
            remediation: "Explicitly list required resources, verbs, and API groups".to_string(),
            severity: Severity::Medium,
            cis_benchmark: Some("5.1.3".to_string()),
            nsa_reference: Some("Authorization".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Authorization,
            title: "Default service account restricted".to_string(),
            description: "Default service account should have minimal permissions".to_string(),
            rationale: "Pods use default SA if none specified; should have minimal access".to_string(),
            remediation: "Do not grant additional permissions to default service account".to_string(),
            severity: Severity::Medium,
            cis_benchmark: Some("5.1.5".to_string()),
            nsa_reference: Some("Authorization".to_string()),
        },

        // Logging & Monitoring
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Logging,
            title: "Audit logging enabled".to_string(),
            description: "Kubernetes audit logging should be enabled".to_string(),
            rationale: "Audit logs provide visibility into cluster activity".to_string(),
            remediation: "Configure --audit-log-path and --audit-policy-file on API server".to_string(),
            severity: Severity::High,
            cis_benchmark: Some("1.2.22".to_string()),
            nsa_reference: Some("Logging".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Logging,
            title: "Audit log retention".to_string(),
            description: "Audit logs should be retained for at least 30 days".to_string(),
            rationale: "Allows investigation of security incidents".to_string(),
            remediation: "Configure --audit-log-maxage=30 or ship logs to external storage".to_string(),
            severity: Severity::Medium,
            cis_benchmark: Some("1.2.23".to_string()),
            nsa_reference: Some("Logging".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Logging,
            title: "Centralized logging".to_string(),
            description: "Container logs should be shipped to centralized logging system".to_string(),
            rationale: "Centralized logs survive container/pod restarts".to_string(),
            remediation: "Deploy logging agent (Fluentd, Fluent Bit, Filebeat) to ship logs".to_string(),
            severity: Severity::Medium,
            cis_benchmark: None,
            nsa_reference: Some("Logging".to_string()),
        },

        // Threat Detection
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::ThreatDetection,
            title: "Runtime security monitoring".to_string(),
            description: "Deploy runtime security tool for threat detection".to_string(),
            rationale: "Detects anomalous behavior and potential attacks at runtime".to_string(),
            remediation: "Deploy Falco, Sysdig, or similar runtime security tool".to_string(),
            severity: Severity::High,
            cis_benchmark: None,
            nsa_reference: Some("Threat Detection".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::ThreatDetection,
            title: "Admission controller for security".to_string(),
            description: "Deploy admission controller to enforce security policies".to_string(),
            rationale: "Prevents deployment of non-compliant workloads".to_string(),
            remediation: "Deploy OPA Gatekeeper, Kyverno, or Pod Security Admission".to_string(),
            severity: Severity::High,
            cis_benchmark: None,
            nsa_reference: Some("Threat Detection".to_string()),
        },

        // Supply Chain Security
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::SupplyChain,
            title: "Image scanning enabled".to_string(),
            description: "Container images should be scanned for vulnerabilities".to_string(),
            rationale: "Identifies known vulnerabilities before deployment".to_string(),
            remediation: "Integrate image scanning in CI/CD pipeline (Trivy, Clair, Snyk)".to_string(),
            severity: Severity::High,
            cis_benchmark: None,
            nsa_reference: Some("Supply Chain".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::SupplyChain,
            title: "Image signature verification".to_string(),
            description: "Container images should be signed and verified".to_string(),
            rationale: "Ensures images come from trusted sources".to_string(),
            remediation: "Use cosign for signing and verification; enforce with admission controller".to_string(),
            severity: Severity::Medium,
            cis_benchmark: None,
            nsa_reference: Some("Supply Chain".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::SupplyChain,
            title: "Private registry only".to_string(),
            description: "Images should only be pulled from trusted private registries".to_string(),
            rationale: "Prevents use of untrusted public images".to_string(),
            remediation: "Configure admission controller to only allow specific registries".to_string(),
            severity: Severity::Medium,
            cis_benchmark: None,
            nsa_reference: Some("Supply Chain".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::SupplyChain,
            title: "No latest tag".to_string(),
            description: "Container images should use specific version tags, not 'latest'".to_string(),
            rationale: "Latest tag can change unexpectedly; specific tags ensure reproducibility".to_string(),
            remediation: "Always use immutable image tags (e.g., sha256 digest or semantic version)".to_string(),
            severity: Severity::Low,
            cis_benchmark: None,
            nsa_reference: Some("Supply Chain".to_string()),
        },

        // Secrets Management
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Secrets,
            title: "Encryption at rest for secrets".to_string(),
            description: "Kubernetes Secrets should be encrypted at rest".to_string(),
            rationale: "Protects secrets stored in etcd".to_string(),
            remediation: "Configure EncryptionConfiguration with KMS or aescbc provider".to_string(),
            severity: Severity::High,
            cis_benchmark: Some("1.2.31".to_string()),
            nsa_reference: Some("Secrets".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Secrets,
            title: "External secrets manager".to_string(),
            description: "Use external secrets manager for sensitive data".to_string(),
            rationale: "External vaults provide better security and rotation".to_string(),
            remediation: "Integrate with HashiCorp Vault, AWS Secrets Manager, or Azure Key Vault".to_string(),
            severity: Severity::Medium,
            cis_benchmark: None,
            nsa_reference: Some("Secrets".to_string()),
        },
        K8sHardeningCheck {
            id: Uuid::new_v4().to_string(),
            category: K8sHardeningCategory::Secrets,
            title: "No secrets in environment variables".to_string(),
            description: "Avoid passing secrets as environment variables".to_string(),
            rationale: "Environment variables can leak in logs and debugging output".to_string(),
            remediation: "Use volume-mounted secrets or secrets store CSI driver".to_string(),
            severity: Severity::Medium,
            cis_benchmark: None,
            nsa_reference: Some("Secrets".to_string()),
        },
    ]
}

/// Get checks grouped by category
pub fn get_checks_by_category() -> Vec<(K8sHardeningCategory, Vec<K8sHardeningCheck>)> {
    let all_checks = get_k8s_hardening_checks();
    let mut grouped: std::collections::HashMap<K8sHardeningCategory, Vec<K8sHardeningCheck>> =
        std::collections::HashMap::new();

    for check in all_checks {
        grouped.entry(check.category).or_default().push(check);
    }

    let mut result: Vec<_> = grouped.into_iter().collect();
    result.sort_by(|a, b| {
        let order_a = K8sHardeningCategory::all().iter().position(|c| *c == a.0).unwrap_or(99);
        let order_b = K8sHardeningCategory::all().iter().position(|c| *c == b.0).unwrap_or(99);
        order_a.cmp(&order_b)
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checks_not_empty() {
        let checks = get_k8s_hardening_checks();
        assert!(!checks.is_empty());
        assert!(checks.len() >= 25);
    }

    #[test]
    fn test_all_categories_covered() {
        let checks = get_k8s_hardening_checks();
        for category in K8sHardeningCategory::all() {
            let count = checks.iter().filter(|c| c.category == category).count();
            assert!(count > 0, "Category {:?} has no checks", category);
        }
    }
}
