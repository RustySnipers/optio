//! Cloud Readiness Assessment
//!
//! Interactive checklist for cloud migration readiness based on
//! industry best practices and the 15-step cloud migration guide.

use crate::infrastructure::models::*;
use uuid::Uuid;

/// Get all cloud readiness checklist items
pub fn get_readiness_checklist() -> Vec<ReadinessCheckItem> {
    vec![
        // Business Alignment
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::BusinessAlignment,
            title: "Executive Sponsorship".to_string(),
            description: "Secure executive sponsorship and establish governance structure for cloud migration".to_string(),
            guidance: Some("Identify C-level sponsor, establish steering committee, define decision-making authority".to_string()),
            priority: 5,
            order: 1,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::BusinessAlignment,
            title: "Business Case Development".to_string(),
            description: "Develop comprehensive business case with ROI analysis and TCO comparison".to_string(),
            guidance: Some("Include direct costs, indirect costs, opportunity costs, and risk considerations".to_string()),
            priority: 5,
            order: 2,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::BusinessAlignment,
            title: "Migration Goals Definition".to_string(),
            description: "Define clear, measurable migration goals and success criteria".to_string(),
            guidance: Some("SMART goals: Specific, Measurable, Achievable, Relevant, Time-bound".to_string()),
            priority: 5,
            order: 3,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::BusinessAlignment,
            title: "Stakeholder Alignment".to_string(),
            description: "Align all stakeholders on migration timeline, expectations, and responsibilities".to_string(),
            guidance: Some("Conduct stakeholder mapping and create communication plan".to_string()),
            priority: 4,
            order: 4,
        },

        // Technical Readiness
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::TechnicalReadiness,
            title: "Application Portfolio Discovery".to_string(),
            description: "Complete inventory of all applications, dependencies, and integrations".to_string(),
            guidance: Some("Use automated discovery tools and manual validation".to_string()),
            priority: 5,
            order: 1,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::TechnicalReadiness,
            title: "Application Assessment (6 Rs)".to_string(),
            description: "Assess each application using the 6 Rs framework: Rehost, Replatform, Repurchase, Refactor, Retire, Retain".to_string(),
            guidance: Some("Consider business value, technical complexity, and dependencies".to_string()),
            priority: 5,
            order: 2,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::TechnicalReadiness,
            title: "Infrastructure Assessment".to_string(),
            description: "Document current infrastructure: compute, storage, network, databases".to_string(),
            guidance: Some("Include specifications, utilization metrics, and performance baselines".to_string()),
            priority: 5,
            order: 3,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::TechnicalReadiness,
            title: "Dependency Mapping".to_string(),
            description: "Map all application and infrastructure dependencies".to_string(),
            guidance: Some("Include internal, external, and third-party dependencies".to_string()),
            priority: 4,
            order: 4,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::TechnicalReadiness,
            title: "Network Architecture Review".to_string(),
            description: "Review and document current network architecture and connectivity requirements".to_string(),
            guidance: Some("Include bandwidth, latency requirements, and VPN/Direct Connect needs".to_string()),
            priority: 4,
            order: 5,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::TechnicalReadiness,
            title: "Cloud Provider Selection".to_string(),
            description: "Evaluate and select appropriate cloud provider(s) based on requirements".to_string(),
            guidance: Some("Consider multi-cloud strategy, vendor lock-in, and regional availability".to_string()),
            priority: 5,
            order: 6,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::TechnicalReadiness,
            title: "Landing Zone Design".to_string(),
            description: "Design cloud landing zone with account structure, networking, and security foundations".to_string(),
            guidance: Some("Use cloud provider best practices (AWS Control Tower, Azure Landing Zones)".to_string()),
            priority: 5,
            order: 7,
        },

        // Security & Compliance
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::SecurityCompliance,
            title: "Compliance Requirements".to_string(),
            description: "Identify all regulatory and compliance requirements (HIPAA, PCI-DSS, SOC 2, GDPR)".to_string(),
            guidance: Some("Map requirements to cloud provider compliance certifications".to_string()),
            priority: 5,
            order: 1,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::SecurityCompliance,
            title: "Security Architecture".to_string(),
            description: "Design cloud security architecture aligned with zero-trust principles".to_string(),
            guidance: Some("Include identity, network, data, and application security layers".to_string()),
            priority: 5,
            order: 2,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::SecurityCompliance,
            title: "Identity & Access Management".to_string(),
            description: "Plan IAM strategy including SSO, MFA, and privileged access management".to_string(),
            guidance: Some("Integrate with existing identity provider where possible".to_string()),
            priority: 5,
            order: 3,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::SecurityCompliance,
            title: "Data Classification".to_string(),
            description: "Classify data by sensitivity and define handling requirements".to_string(),
            guidance: Some("Include PII, PHI, financial data, and intellectual property".to_string()),
            priority: 4,
            order: 4,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::SecurityCompliance,
            title: "Encryption Strategy".to_string(),
            description: "Define encryption requirements for data at rest and in transit".to_string(),
            guidance: Some("Consider key management: cloud-managed vs. customer-managed keys".to_string()),
            priority: 4,
            order: 5,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::SecurityCompliance,
            title: "Security Monitoring Plan".to_string(),
            description: "Plan cloud security monitoring, SIEM integration, and incident response".to_string(),
            guidance: Some("Include cloud-native security services and third-party tools".to_string()),
            priority: 4,
            order: 6,
        },

        // Operational Readiness
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::OperationalReadiness,
            title: "Operations Model".to_string(),
            description: "Define cloud operations model: CloudOps, DevOps, SRE".to_string(),
            guidance: Some("Consider shared responsibility model with cloud provider".to_string()),
            priority: 4,
            order: 1,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::OperationalReadiness,
            title: "Monitoring & Observability".to_string(),
            description: "Plan monitoring, logging, and observability strategy".to_string(),
            guidance: Some("Include metrics, logs, traces, and alerting".to_string()),
            priority: 4,
            order: 2,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::OperationalReadiness,
            title: "Backup & Disaster Recovery".to_string(),
            description: "Design backup and disaster recovery strategy for cloud".to_string(),
            guidance: Some("Define RPO/RTO requirements and test recovery procedures".to_string()),
            priority: 5,
            order: 3,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::OperationalReadiness,
            title: "Change Management".to_string(),
            description: "Establish change management process for cloud environment".to_string(),
            guidance: Some("Include approval workflows and rollback procedures".to_string()),
            priority: 3,
            order: 4,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::OperationalReadiness,
            title: "Automation Strategy".to_string(),
            description: "Define Infrastructure as Code (IaC) and automation approach".to_string(),
            guidance: Some("Select tools: Terraform, CloudFormation, Pulumi, Ansible".to_string()),
            priority: 4,
            order: 5,
        },

        // Financial Planning
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::FinancialPlanning,
            title: "Cost Baseline".to_string(),
            description: "Establish current infrastructure cost baseline".to_string(),
            guidance: Some("Include hardware, software, facilities, and personnel costs".to_string()),
            priority: 5,
            order: 1,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::FinancialPlanning,
            title: "Cloud Cost Estimation".to_string(),
            description: "Estimate cloud costs using provider pricing calculators".to_string(),
            guidance: Some("Consider compute, storage, network, and managed services".to_string()),
            priority: 5,
            order: 2,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::FinancialPlanning,
            title: "FinOps Practices".to_string(),
            description: "Establish FinOps practices for cloud cost optimization".to_string(),
            guidance: Some("Include tagging strategy, budgets, alerts, and reserved capacity planning".to_string()),
            priority: 4,
            order: 3,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::FinancialPlanning,
            title: "Migration Budget".to_string(),
            description: "Define migration project budget including tools, training, and consulting".to_string(),
            guidance: Some("Include contingency for unexpected issues".to_string()),
            priority: 4,
            order: 4,
        },

        // People & Process
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::PeopleProcess,
            title: "Skills Assessment".to_string(),
            description: "Assess current team cloud skills and identify gaps".to_string(),
            guidance: Some("Consider certifications: AWS, Azure, GCP, Kubernetes".to_string()),
            priority: 4,
            order: 1,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::PeopleProcess,
            title: "Training Plan".to_string(),
            description: "Develop cloud training and certification plan".to_string(),
            guidance: Some("Include hands-on labs, sandbox environments, and mentoring".to_string()),
            priority: 4,
            order: 2,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::PeopleProcess,
            title: "Migration Team".to_string(),
            description: "Establish dedicated migration team with clear roles".to_string(),
            guidance: Some("Include architects, engineers, security, and project management".to_string()),
            priority: 4,
            order: 3,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::PeopleProcess,
            title: "Organizational Change".to_string(),
            description: "Plan organizational change management for cloud adoption".to_string(),
            guidance: Some("Address culture, processes, and resistance to change".to_string()),
            priority: 3,
            order: 4,
        },

        // Data Management
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::DataManagement,
            title: "Data Inventory".to_string(),
            description: "Complete inventory of all data stores and volumes".to_string(),
            guidance: Some("Include databases, file shares, object storage, and archives".to_string()),
            priority: 5,
            order: 1,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::DataManagement,
            title: "Data Migration Strategy".to_string(),
            description: "Define data migration approach: online, offline, or hybrid".to_string(),
            guidance: Some("Consider tools: AWS DMS, Azure Migrate, native replication".to_string()),
            priority: 5,
            order: 2,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::DataManagement,
            title: "Data Residency".to_string(),
            description: "Identify data residency requirements and select appropriate regions".to_string(),
            guidance: Some("Consider regulatory requirements for data location".to_string()),
            priority: 4,
            order: 3,
        },
        ReadinessCheckItem {
            id: Uuid::new_v4().to_string(),
            category: ReadinessCategory::DataManagement,
            title: "Data Validation".to_string(),
            description: "Plan data validation and integrity verification post-migration".to_string(),
            guidance: Some("Include checksums, record counts, and application testing".to_string()),
            priority: 4,
            order: 4,
        },
    ]
}

/// Get checklist items grouped by category
pub fn get_checklist_by_category() -> Vec<(ReadinessCategory, Vec<ReadinessCheckItem>)> {
    let all_items = get_readiness_checklist();
    let mut grouped: std::collections::HashMap<ReadinessCategory, Vec<ReadinessCheckItem>> =
        std::collections::HashMap::new();

    for item in all_items {
        grouped.entry(item.category).or_default().push(item);
    }

    let mut result: Vec<_> = grouped.into_iter().collect();
    result.sort_by(|a, b| {
        let order_a = ReadinessCategory::all().iter().position(|c| *c == a.0).unwrap_or(99);
        let order_b = ReadinessCategory::all().iter().position(|c| *c == b.0).unwrap_or(99);
        order_a.cmp(&order_b)
    });

    for (_, items) in &mut result {
        items.sort_by_key(|i| i.order);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checklist_not_empty() {
        let checklist = get_readiness_checklist();
        assert!(!checklist.is_empty());
        assert!(checklist.len() >= 30); // Should have at least 30 items
    }

    #[test]
    fn test_all_categories_covered() {
        let checklist = get_readiness_checklist();
        for category in ReadinessCategory::all() {
            let count = checklist.iter().filter(|c| c.category == category).count();
            assert!(count > 0, "Category {:?} has no items", category);
        }
    }
}
