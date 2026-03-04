# Document Templates

## Technical Spec Template

```markdown
# [Feature Name] Technical Spec

**Author**: [Your Name]
**Date**: [Date]
**Status**: [Draft/Review/Approved]

## Overview
[1-2 paragraphs describing what this document covers]

## Background
[Context and motivation]

## Goals
- Goal 1
- Goal 2

## Non-Goals
[What we're not doing]

## Detailed Design
[Technical details]

## Alternatives Considered
[Other approaches and why we didn't choose them]

## Timeline
- Week 1: ...
- Week 2: ...

## Open Questions
- Question 1
- Question 2
```

## Architecture Document Template

```markdown
# System Architecture

## Overview
High-level system description

## Architecture Diagram
[Insert Mermaid diagram]

## Components
### Component 1
- Responsibility: ...
- Technology: ...
- Interfaces: ...

## Data Flow
[How data moves through the system]

## Key Design Decisions
### Decision 1
- Context: ...
- Options considered: ...
- Decision: ...
- Rationale: ...

## Technology Stack
- Frontend: ...
- Backend: ...
- Database: ...
- Infrastructure: ...

## Security
[Authentication, authorization, data protection]

## Monitoring
[Metrics, logs, tracing]

## Disaster Recovery
[Backup and recovery procedures]
```

## Runbook Template

```markdown
# [Service Name] Runbook

## Service Overview
What this service does

## Dependencies
- Service A
- Service B
- Database X

## Deployment
### Deploy
```bash
./deploy.sh production
```

### Rollback
```bash
./rollback.sh
```

## Monitoring
### Key Metrics
- Request rate
- Error rate
- Latency

### Dashboards
- [Production Dashboard](link)

## Common Issues
### Issue: High latency
**Symptoms**: Response time > 1s
**Diagnosis**: Check database connection pool
**Resolution**: Restart service or scale up

## Troubleshooting
```bash
kubectl logs -f deployment/service-name
```

## Emergency Contacts
- On-call: [PagerDuty](link)
- Team Slack: #team-name
```

## API Documentation Template

```markdown
# API Documentation

## Authentication
All requests require authentication:
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.example.com/endpoint
```

## Endpoints

### [Endpoint Name]
```
METHOD /api/v1/resource
```

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| id   | integer | Yes | Resource ID |

**Example Request**:
```bash
curl -X GET "https://api.example.com/api/v1/resource/1" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**Example Response**:
```json
{
  "id": 1,
  "name": "Example"
}
```

**Error Responses**:
| Status | Description |
|--------|-------------|
| 400   | Bad Request |
| 401   | Unauthorized |
| 500   | Server Error |
```

## Changelog Template

```markdown
# Changelog

## [1.2.0] - 2024-01-15

### Added
- New feature X

### Changed
- Improved performance of Z

### Fixed
- Bug where user couldn't login

### Deprecated
- Old API endpoint /v1/users (use /v2/users)

### Removed
- Legacy authentication method

### Security
- Fixed XSS vulnerability
```
