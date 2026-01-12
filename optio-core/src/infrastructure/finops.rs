//! FinOps Calculator Module
//!
//! Cloud cost projection and optimization analysis for infrastructure migrations.
//! Provides TCO calculations, cost comparisons, and optimization recommendations.

use super::models::*;

/// Default pricing data for major cloud providers (per hour, simplified)
pub struct CloudPricing {
    pub compute_per_vcpu: f64,
    pub memory_per_gb: f64,
    pub storage_per_gb_month: f64,
    pub egress_per_gb: f64,
    pub managed_db_multiplier: f64,
    pub k8s_cluster_base: f64,
}

impl CloudPricing {
    /// Get pricing for a cloud provider (simplified baseline pricing)
    pub fn for_provider(provider: &CloudProvider) -> Self {
        match provider {
            CloudProvider::AWS => Self {
                compute_per_vcpu: 0.0416,      // ~$30/month per vCPU
                memory_per_gb: 0.005,           // ~$3.6/month per GB
                storage_per_gb_month: 0.10,     // EBS gp3
                egress_per_gb: 0.09,            // First 10TB
                managed_db_multiplier: 1.5,     // RDS premium
                k8s_cluster_base: 0.10,         // EKS per hour
            },
            CloudProvider::Azure => Self {
                compute_per_vcpu: 0.04,
                memory_per_gb: 0.0048,
                storage_per_gb_month: 0.095,
                egress_per_gb: 0.087,
                managed_db_multiplier: 1.45,
                k8s_cluster_base: 0.10,         // AKS control plane
            },
            CloudProvider::GCP => Self {
                compute_per_vcpu: 0.038,
                memory_per_gb: 0.0045,
                storage_per_gb_month: 0.08,
                egress_per_gb: 0.085,
                managed_db_multiplier: 1.4,
                k8s_cluster_base: 0.10,         // GKE autopilot base
            },
        }
    }
}

/// Calculate monthly cost for a resource
pub fn calculate_resource_cost(
    resource: &ResourceCostEstimate,
    provider: &CloudProvider,
) -> f64 {
    let pricing = CloudPricing::for_provider(provider);
    let hours_per_month = 730.0;

    match resource.resource_type {
        ResourceType::VirtualMachine => {
            let compute_cost = resource.quantity as f64
                * resource.specs.vcpus.unwrap_or(2) as f64
                * pricing.compute_per_vcpu
                * hours_per_month;
            let memory_cost = resource.quantity as f64
                * resource.specs.memory_gb.unwrap_or(4.0)
                * pricing.memory_per_gb
                * hours_per_month;
            compute_cost + memory_cost
        }
        ResourceType::Container => {
            // Containers typically use fractional resources
            let compute_cost = resource.quantity as f64
                * resource.specs.vcpus.unwrap_or(1) as f64
                * pricing.compute_per_vcpu
                * hours_per_month
                * 0.5; // Container overhead factor
            let memory_cost = resource.quantity as f64
                * resource.specs.memory_gb.unwrap_or(2.0)
                * pricing.memory_per_gb
                * hours_per_month;
            compute_cost + memory_cost
        }
        ResourceType::Database => {
            let base_compute = resource.quantity as f64
                * resource.specs.vcpus.unwrap_or(2) as f64
                * pricing.compute_per_vcpu
                * hours_per_month;
            let base_memory = resource.quantity as f64
                * resource.specs.memory_gb.unwrap_or(8.0)
                * pricing.memory_per_gb
                * hours_per_month;
            let storage_cost = resource.specs.storage_gb.unwrap_or(100.0)
                * pricing.storage_per_gb_month
                * 1.5; // Database storage premium
            (base_compute + base_memory) * pricing.managed_db_multiplier + storage_cost
        }
        ResourceType::Storage => {
            resource.specs.storage_gb.unwrap_or(100.0) * pricing.storage_per_gb_month
        }
        ResourceType::Network => {
            // Network costs based on estimated egress
            resource.specs.bandwidth_gbps.unwrap_or(1.0)
                * 1000.0 // GB per Gbps per month (simplified)
                * pricing.egress_per_gb
                * 0.3 // Typical utilization
        }
        ResourceType::Kubernetes => {
            let cluster_cost = pricing.k8s_cluster_base * hours_per_month;
            let node_cost = resource.quantity as f64
                * resource.specs.vcpus.unwrap_or(4) as f64
                * pricing.compute_per_vcpu
                * hours_per_month;
            let memory_cost = resource.quantity as f64
                * resource.specs.memory_gb.unwrap_or(16.0)
                * pricing.memory_per_gb
                * hours_per_month;
            cluster_cost + node_cost + memory_cost
        }
        ResourceType::Serverless => {
            // Serverless pricing is request-based, estimate based on invocations
            let invocations_per_month = resource.quantity as f64 * 1_000_000.0;
            let compute_time_ms = 200.0; // Average execution time
            let memory_mb = resource.specs.memory_gb.unwrap_or(0.5) * 1024.0;

            // AWS Lambda-style pricing
            let request_cost = invocations_per_month * 0.0000002;
            let compute_cost = invocations_per_month
                * (compute_time_ms / 1000.0)
                * (memory_mb / 1024.0)
                * 0.0000166667;
            request_cost + compute_cost
        }
        ResourceType::LoadBalancer => {
            // Application Load Balancer pricing
            let base_cost = 0.0225 * hours_per_month; // ALB hourly
            let lcu_cost = resource.quantity as f64 * 0.008 * hours_per_month;
            base_cost + lcu_cost
        }
        ResourceType::Other => {
            // Generic calculation
            resource.quantity as f64 * 50.0 // $50/month baseline
        }
    }
}

/// Generate a complete FinOps analysis
pub fn generate_finops_analysis(
    current_costs: &OnPremiseCosts,
    resources: &[ResourceCostEstimate],
    provider: &CloudProvider,
    migration_strategy: &MigrationStrategy,
) -> FinOpsAnalysis {
    // Calculate cloud costs
    let mut resource_estimates = Vec::new();
    let mut total_monthly_cloud = 0.0;

    for resource in resources {
        let monthly_cost = calculate_resource_cost(resource, provider);
        let mut estimate = resource.clone();
        estimate.monthly_cost = monthly_cost;
        total_monthly_cloud += monthly_cost;
        resource_estimates.push(estimate);
    }

    // Apply strategy-based adjustments
    let strategy_multiplier = match migration_strategy {
        MigrationStrategy::Rehost => 1.0,           // Lift and shift, no optimization
        MigrationStrategy::Replatform => 0.85,      // Some optimization
        MigrationStrategy::Refactor => 0.70,        // Cloud-native optimization
        MigrationStrategy::Repurchase => 0.90,      // SaaS replacement
        MigrationStrategy::Retire => 0.0,           // Removing
        MigrationStrategy::Retain => 1.0,           // Keeping as-is
    };

    let optimized_monthly = total_monthly_cloud * strategy_multiplier;

    // Calculate annual costs
    let annual_cloud_cost = optimized_monthly * 12.0;
    let current_annual = current_costs.annual_total();

    // Calculate savings
    let annual_savings = current_annual - annual_cloud_cost;
    let savings_percentage = if current_annual > 0.0 {
        (annual_savings / current_annual) * 100.0
    } else {
        0.0
    };

    // Estimate migration costs
    let migration_cost = estimate_migration_cost(resources, migration_strategy);

    // Calculate ROI and payback
    let roi_percentage = if migration_cost > 0.0 {
        ((annual_savings * 3.0 - migration_cost) / migration_cost) * 100.0
    } else {
        0.0
    };

    let payback_months = if annual_savings > 0.0 {
        (migration_cost / (annual_savings / 12.0)) as u32
    } else {
        0
    };

    // Generate recommendations
    let recommendations = generate_cost_recommendations(
        &resource_estimates,
        provider,
        migration_strategy,
    );

    FinOpsAnalysis {
        id: uuid::Uuid::new_v4().to_string(),
        client_id: String::new(), // Set by caller
        analysis_date: chrono::Utc::now().to_rfc3339(),
        target_provider: provider.clone(),
        migration_strategy: migration_strategy.clone(),
        current_monthly_cost: current_costs.monthly_total(),
        projected_monthly_cost: optimized_monthly,
        estimated_savings_percentage: savings_percentage,
        migration_cost_estimate: migration_cost,
        roi_months: payback_months,
        resource_breakdown: resource_estimates,
        recommendations,
        assumptions: vec![
            "Pricing based on on-demand rates; reserved instances can reduce costs by 30-60%".to_string(),
            "Network egress estimated at 30% of provisioned bandwidth".to_string(),
            "Managed services include high availability configuration".to_string(),
            "Migration costs include planning, execution, and 3-month parallel run".to_string(),
        ],
    }
}

/// Estimate migration project costs
fn estimate_migration_cost(
    resources: &[ResourceCostEstimate],
    strategy: &MigrationStrategy,
) -> f64 {
    let resource_count = resources.len() as f64;
    let complexity_factor = resources.iter()
        .map(|r| match r.resource_type {
            ResourceType::Database => 3.0,
            ResourceType::Kubernetes => 2.5,
            ResourceType::VirtualMachine => 1.0,
            ResourceType::Container => 1.5,
            ResourceType::Serverless => 2.0,
            _ => 1.0,
        })
        .sum::<f64>() / resource_count.max(1.0);

    let base_cost = match strategy {
        MigrationStrategy::Rehost => 5_000.0,
        MigrationStrategy::Replatform => 15_000.0,
        MigrationStrategy::Refactor => 50_000.0,
        MigrationStrategy::Repurchase => 10_000.0,
        MigrationStrategy::Retire => 2_000.0,
        MigrationStrategy::Retain => 0.0,
    };

    base_cost * complexity_factor * (resource_count / 10.0).max(1.0)
}

/// Generate cost optimization recommendations
fn generate_cost_recommendations(
    resources: &[ResourceCostEstimate],
    provider: &CloudProvider,
    strategy: &MigrationStrategy,
) -> Vec<CostRecommendation> {
    let mut recommendations = Vec::new();

    // Reserved instance recommendation
    let total_monthly: f64 = resources.iter().map(|r| r.monthly_cost).sum();
    if total_monthly > 1000.0 {
        recommendations.push(CostRecommendation {
            category: "Commitment Discounts".to_string(),
            title: format!("Consider {} Reserved Instances", match provider {
                CloudProvider::AWS => "AWS",
                CloudProvider::Azure => "Azure",
                CloudProvider::GCP => "GCP Committed Use",
            }),
            description: "For predictable workloads, 1-3 year commitments can reduce compute costs by 30-60%".to_string(),
            estimated_savings: total_monthly * 0.35 * 12.0,
            effort: "Low".to_string(),
            priority: 1,
        });
    }

    // Right-sizing recommendation
    let vm_resources: Vec<_> = resources.iter()
        .filter(|r| matches!(r.resource_type, ResourceType::VirtualMachine | ResourceType::Container))
        .collect();
    if !vm_resources.is_empty() {
        recommendations.push(CostRecommendation {
            category: "Right-sizing".to_string(),
            title: "Implement resource right-sizing".to_string(),
            description: "Analyze actual utilization and resize over-provisioned instances. Typical organizations are 40% over-provisioned.".to_string(),
            estimated_savings: vm_resources.iter().map(|r| r.monthly_cost).sum::<f64>() * 0.25 * 12.0,
            effort: "Medium".to_string(),
            priority: 2,
        });
    }

    // Spot/Preemptible recommendation
    recommendations.push(CostRecommendation {
        category: "Spot Instances".to_string(),
        title: format!("Use {} for fault-tolerant workloads", match provider {
            CloudProvider::AWS => "AWS Spot Instances",
            CloudProvider::Azure => "Azure Spot VMs",
            CloudProvider::GCP => "GCP Preemptible VMs",
        }),
        description: "Stateless and fault-tolerant workloads can use spot/preemptible instances for 60-90% cost reduction".to_string(),
        estimated_savings: total_monthly * 0.15 * 12.0,
        effort: "Medium".to_string(),
        priority: 3,
    });

    // Storage optimization
    let storage_resources: Vec<_> = resources.iter()
        .filter(|r| matches!(r.resource_type, ResourceType::Storage))
        .collect();
    if !storage_resources.is_empty() {
        recommendations.push(CostRecommendation {
            category: "Storage Optimization".to_string(),
            title: "Implement storage lifecycle policies".to_string(),
            description: "Move infrequently accessed data to cheaper storage tiers (Glacier, Cool, Archive)".to_string(),
            estimated_savings: storage_resources.iter().map(|r| r.monthly_cost).sum::<f64>() * 0.40 * 12.0,
            effort: "Low".to_string(),
            priority: 2,
        });
    }

    // Serverless recommendation for refactor strategy
    if matches!(strategy, MigrationStrategy::Refactor) {
        recommendations.push(CostRecommendation {
            category: "Architecture".to_string(),
            title: "Evaluate serverless architecture".to_string(),
            description: "Consider AWS Lambda, Azure Functions, or Cloud Run for variable workloads to optimize cost-per-request".to_string(),
            estimated_savings: total_monthly * 0.20 * 12.0,
            effort: "High".to_string(),
            priority: 4,
        });
    }

    // Kubernetes optimization
    let k8s_resources: Vec<_> = resources.iter()
        .filter(|r| matches!(r.resource_type, ResourceType::Kubernetes))
        .collect();
    if !k8s_resources.is_empty() {
        recommendations.push(CostRecommendation {
            category: "Kubernetes".to_string(),
            title: "Enable cluster autoscaling".to_string(),
            description: "Implement Horizontal Pod Autoscaler (HPA) and Cluster Autoscaler to match capacity with demand".to_string(),
            estimated_savings: k8s_resources.iter().map(|r| r.monthly_cost).sum::<f64>() * 0.30 * 12.0,
            effort: "Medium".to_string(),
            priority: 2,
        });
    }

    // Sort by priority
    recommendations.sort_by_key(|r| r.priority);
    recommendations
}

/// Get default resource templates for common workload types
pub fn get_resource_templates() -> Vec<ResourceTemplate> {
    vec![
        ResourceTemplate {
            name: "Small Web Application".to_string(),
            description: "2-tier web app with database".to_string(),
            resources: vec![
                ResourceCostEstimate {
                    resource_type: ResourceType::VirtualMachine,
                    name: "Web Servers".to_string(),
                    quantity: 2,
                    specs: ResourceSpecs {
                        vcpus: Some(2),
                        memory_gb: Some(4.0),
                        storage_gb: Some(50.0),
                        bandwidth_gbps: None,
                        iops: None,
                    },
                    monthly_cost: 0.0,
                    notes: Some("Auto-scaling group".to_string()),
                },
                ResourceCostEstimate {
                    resource_type: ResourceType::Database,
                    name: "PostgreSQL Database".to_string(),
                    quantity: 1,
                    specs: ResourceSpecs {
                        vcpus: Some(2),
                        memory_gb: Some(8.0),
                        storage_gb: Some(100.0),
                        bandwidth_gbps: None,
                        iops: Some(3000),
                    },
                    monthly_cost: 0.0,
                    notes: Some("Multi-AZ deployment".to_string()),
                },
                ResourceCostEstimate {
                    resource_type: ResourceType::LoadBalancer,
                    name: "Application Load Balancer".to_string(),
                    quantity: 1,
                    specs: ResourceSpecs::default(),
                    monthly_cost: 0.0,
                    notes: None,
                },
            ],
        },
        ResourceTemplate {
            name: "Kubernetes Microservices".to_string(),
            description: "Container-based microservices platform".to_string(),
            resources: vec![
                ResourceCostEstimate {
                    resource_type: ResourceType::Kubernetes,
                    name: "K8s Worker Nodes".to_string(),
                    quantity: 3,
                    specs: ResourceSpecs {
                        vcpus: Some(4),
                        memory_gb: Some(16.0),
                        storage_gb: Some(100.0),
                        bandwidth_gbps: None,
                        iops: None,
                    },
                    monthly_cost: 0.0,
                    notes: Some("Production cluster".to_string()),
                },
                ResourceCostEstimate {
                    resource_type: ResourceType::Database,
                    name: "Managed PostgreSQL".to_string(),
                    quantity: 1,
                    specs: ResourceSpecs {
                        vcpus: Some(4),
                        memory_gb: Some(16.0),
                        storage_gb: Some(500.0),
                        bandwidth_gbps: None,
                        iops: Some(6000),
                    },
                    monthly_cost: 0.0,
                    notes: Some("HA configuration".to_string()),
                },
                ResourceCostEstimate {
                    resource_type: ResourceType::Storage,
                    name: "Object Storage".to_string(),
                    quantity: 1,
                    specs: ResourceSpecs {
                        vcpus: None,
                        memory_gb: None,
                        storage_gb: Some(1000.0),
                        bandwidth_gbps: None,
                        iops: None,
                    },
                    monthly_cost: 0.0,
                    notes: Some("S3/Blob/GCS".to_string()),
                },
            ],
        },
        ResourceTemplate {
            name: "Data Analytics Platform".to_string(),
            description: "Big data processing and analytics".to_string(),
            resources: vec![
                ResourceCostEstimate {
                    resource_type: ResourceType::VirtualMachine,
                    name: "Analytics Cluster Nodes".to_string(),
                    quantity: 5,
                    specs: ResourceSpecs {
                        vcpus: Some(8),
                        memory_gb: Some(32.0),
                        storage_gb: Some(500.0),
                        bandwidth_gbps: Some(10.0),
                        iops: None,
                    },
                    monthly_cost: 0.0,
                    notes: Some("Spark/Hadoop cluster".to_string()),
                },
                ResourceCostEstimate {
                    resource_type: ResourceType::Storage,
                    name: "Data Lake Storage".to_string(),
                    quantity: 1,
                    specs: ResourceSpecs {
                        vcpus: None,
                        memory_gb: None,
                        storage_gb: Some(10000.0),
                        bandwidth_gbps: None,
                        iops: None,
                    },
                    monthly_cost: 0.0,
                    notes: Some("S3/ADLS/GCS".to_string()),
                },
                ResourceCostEstimate {
                    resource_type: ResourceType::Database,
                    name: "Data Warehouse".to_string(),
                    quantity: 1,
                    specs: ResourceSpecs {
                        vcpus: Some(16),
                        memory_gb: Some(64.0),
                        storage_gb: Some(2000.0),
                        bandwidth_gbps: None,
                        iops: Some(10000),
                    },
                    monthly_cost: 0.0,
                    notes: Some("Redshift/Synapse/BigQuery".to_string()),
                },
            ],
        },
    ]
}

/// Resource template for quick estimation
#[derive(Debug, Clone)]
pub struct ResourceTemplate {
    pub name: String,
    pub description: String,
    pub resources: Vec<ResourceCostEstimate>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_vm_cost() {
        let resource = ResourceCostEstimate {
            resource_type: ResourceType::VirtualMachine,
            name: "Test VM".to_string(),
            quantity: 1,
            specs: ResourceSpecs {
                vcpus: Some(4),
                memory_gb: Some(16.0),
                storage_gb: None,
                bandwidth_gbps: None,
                iops: None,
            },
            monthly_cost: 0.0,
            notes: None,
        };

        let cost = calculate_resource_cost(&resource, &CloudProvider::AWS);
        assert!(cost > 0.0);
        assert!(cost < 500.0); // Sanity check
    }

    #[test]
    fn test_finops_analysis() {
        let current_costs = OnPremiseCosts {
            hardware_monthly: 5000.0,
            software_licensing_monthly: 2000.0,
            datacenter_monthly: 1500.0,
            personnel_monthly: 8000.0,
            maintenance_monthly: 1000.0,
            power_cooling_monthly: 500.0,
            network_monthly: 300.0,
        };

        let resources = vec![
            ResourceCostEstimate {
                resource_type: ResourceType::VirtualMachine,
                name: "App Servers".to_string(),
                quantity: 4,
                specs: ResourceSpecs {
                    vcpus: Some(4),
                    memory_gb: Some(16.0),
                    storage_gb: Some(100.0),
                    bandwidth_gbps: None,
                    iops: None,
                },
                monthly_cost: 0.0,
                notes: None,
            },
        ];

        let analysis = generate_finops_analysis(
            &current_costs,
            &resources,
            &CloudProvider::AWS,
            &MigrationStrategy::Replatform,
        );

        assert!(!analysis.id.is_empty());
        assert!(!analysis.recommendations.is_empty());
        assert!(analysis.projected_monthly_cost > 0.0);
    }
}
