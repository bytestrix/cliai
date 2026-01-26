# CLIAI Pro - Google Cloud Setup Guide

## Overview

This guide walks through setting up the CLIAI Pro backend infrastructure on Google Cloud Platform.

## Prerequisites

- Google Cloud account with billing enabled
- `gcloud` CLI installed and configured
- Docker installed locally
- Node.js 18+ and Python 3.9+ for development

## Phase 1: Project Setup

### 1. Create GCP Project

```bash
# Create new project
gcloud projects create cliai-pro-backend --name="CLIAI Pro Backend"

# Set as default project
gcloud config set project cliai-pro-backend

# Enable required APIs
gcloud services enable \
  container.googleapis.com \
  sqladmin.googleapis.com \
  cloudbuild.googleapis.com \
  secretmanager.googleapis.com \
  monitoring.googleapis.com \
  logging.googleapis.com
```

### 2. Set Up Authentication & Security

```bash
# Create service account for the application
gcloud iam service-accounts create cliai-backend \
  --display-name="CLIAI Backend Service Account"

# Grant necessary permissions
gcloud projects add-iam-policy-binding cliai-pro-backend \
  --member="serviceAccount:cliai-backend@cliai-pro-backend.iam.gserviceaccount.com" \
  --role="roles/cloudsql.client"

gcloud projects add-iam-policy-binding cliai-pro-backend \
  --member="serviceAccount:cliai-backend@cliai-pro-backend.iam.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"
```

### 3. Database Setup (Cloud SQL)

```bash
# Create PostgreSQL instance
gcloud sql instances create cliai-db \
  --database-version=POSTGRES_15 \
  --tier=db-f1-micro \
  --region=us-central1 \
  --storage-type=SSD \
  --storage-size=10GB \
  --backup-start-time=03:00

# Create database
gcloud sql databases create cliai_pro --instance=cliai-db

# Create database user
gcloud sql users create cliai_user \
  --instance=cliai-db \
  --password=SECURE_PASSWORD_HERE
```

### 4. Kubernetes Cluster Setup

```bash
# Create GKE cluster
gcloud container clusters create cliai-cluster \
  --zone=us-central1-a \
  --num-nodes=2 \
  --machine-type=e2-medium \
  --enable-autoscaling \
  --min-nodes=1 \
  --max-nodes=5 \
  --enable-autorepair \
  --enable-autoupgrade

# Get cluster credentials
gcloud container clusters get-credentials cliai-cluster --zone=us-central1-a
```

## Phase 2: Application Deployment

### 1. Backend API Structure

```
backend/
├── auth-service/          # Authentication & user management
│   ├── Dockerfile
│   ├── package.json
│   └── src/
├── core-api/             # Main CLIAI Pro API
│   ├── Dockerfile
│   ├── package.json
│   └── src/
├── ai-proxy/             # AI model proxy service
│   ├── Dockerfile
│   ├── requirements.txt
│   └── src/
└── k8s/                  # Kubernetes manifests
    ├── auth-service.yaml
    ├── core-api.yaml
    ├── ai-proxy.yaml
    └── ingress.yaml
```

### 2. Environment Configuration

```bash
# Store secrets in Secret Manager
gcloud secrets create db-password --data-file=db_password.txt
gcloud secrets create openai-api-key --data-file=openai_key.txt
gcloud secrets create anthropic-api-key --data-file=anthropic_key.txt
gcloud secrets create jwt-secret --data-file=jwt_secret.txt
```

### 3. Deploy Services

```bash
# Build and push Docker images
gcloud builds submit --tag gcr.io/cliai-pro-backend/auth-service ./auth-service
gcloud builds submit --tag gcr.io/cliai-pro-backend/core-api ./core-api
gcloud builds submit --tag gcr.io/cliai-pro-backend/ai-proxy ./ai-proxy

# Deploy to Kubernetes
kubectl apply -f k8s/
```

## Phase 3: Domain & SSL

### 1. Domain Setup

```bash
# Reserve static IP
gcloud compute addresses create cliai-pro-ip --global

# Get the IP address
gcloud compute addresses describe cliai-pro-ip --global
```

### 2. SSL Certificate

```bash
# Create managed SSL certificate
gcloud compute ssl-certificates create cliai-pro-ssl \
  --domains=api.cliai.pro
```

## Phase 4: Monitoring & Logging

### 1. Set Up Monitoring

```bash
# Enable monitoring
gcloud services enable monitoring.googleapis.com

# Create uptime checks
gcloud alpha monitoring uptime create cliai-api-check \
  --hostname=api.cliai.pro \
  --path=/health
```

### 2. Set Up Alerting

```bash
# Create notification channel (email)
gcloud alpha monitoring channels create \
  --display-name="CLIAI Alerts" \
  --type=email \
  --channel-labels=email_address=alerts@cliai.pro
```

## Cost Optimization

### Estimated Monthly Costs (USD)

- **GKE Cluster**: $50-150 (2-5 nodes, e2-medium)
- **Cloud SQL**: $15-30 (db-f1-micro with 10GB)
- **Load Balancer**: $18 (global)
- **Storage & Networking**: $10-20
- **AI API Costs**: Variable (pay-per-use)

**Total**: ~$100-220/month for production-ready setup

### Cost Reduction Tips

1. **Use Preemptible Nodes**: 60-80% cost reduction for non-critical workloads
2. **Auto-scaling**: Scale down during low usage
3. **Committed Use Discounts**: 57% discount for 1-year commitment
4. **Regional Persistent Disks**: Cheaper than SSD for non-critical data

## Security Best Practices

1. **Network Security**:
   - Private GKE cluster
   - VPC firewall rules
   - Cloud Armor for DDoS protection

2. **Data Security**:
   - Encryption at rest and in transit
   - Secret Manager for sensitive data
   - Regular security scans

3. **Access Control**:
   - IAM roles and permissions
   - Service account keys rotation
   - Audit logging

## Backup & Disaster Recovery

```bash
# Automated database backups
gcloud sql instances patch cliai-db \
  --backup-start-time=03:00 \
  --retained-backups-count=7

# Cross-region backup
gcloud sql backups create --instance=cliai-db
```

## Next Steps

1. **Set up CI/CD pipeline** with Cloud Build
2. **Implement rate limiting** and API quotas
3. **Add caching layer** with Redis
4. **Set up staging environment**
5. **Configure custom domain** and DNS

## Support Resources

- [GCP Documentation](https://cloud.google.com/docs)
- [GKE Best Practices](https://cloud.google.com/kubernetes-engine/docs/best-practices)
- [Cloud SQL Performance](https://cloud.google.com/sql/docs/postgres/optimize-performance)