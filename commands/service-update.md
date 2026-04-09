---
description: "Update all services, diagnose and fix stuck pods and Tilt issues"
arguments: []
user_invocable: true
---

# Service Update - Full Local Environment Refresh

Update all git repositories and diagnose/fix any infrastructure issues in the local Kind + Tilt environment.

## Phase 1: Update All Repositories

Run the update script to pull latest changes for all services:

```bash
bash <project-root>/scripts/update-all.sh
```

Capture the output and note:
- **Failed repos** - repos where `git pull --ff-only` failed (diverged branches)
- **Stashed repos** - repos with local changes that were stashed
- **Total updated** - count of successfully updated repos

Present the summary to the user.

---

## Phase 2: Health Check - Pod Status

After updating, check all pod statuses:

```bash
kubectl get pods
```

Categorize pods into:
- **Running** - healthy
- **CrashLoopBackOff** - crashing repeatedly
- **Error** - failed
- **Pending** - waiting for resources
- **ContainerCreating** - stuck creating

If ALL pods are Running, skip to Phase 6 (Verify).

---

## Phase 3: Diagnose Tilt Uncategorized Resources

If pods are stuck in Pending or ContainerCreating, check if Tilt's uncategorized resources failed:

```bash
tilt get uiresources uncategorized -o jsonpath='{.status.buildHistory[0].error}'
```

If the output is `resource name may not be empty`, this is the **uncategorized resources bug**.

### Confirm by checking resources:

```bash
kubectl get configmap,pv,pvc -n default
```

If only `kube-root-ca.crt` exists (no app ConfigMaps, no PVs, no PVCs), apply the fix:

### Fix: Manually apply infrastructure resources

```bash
cd <project-root>/backend-services

# Step 1: Postgres PV, PVC and ConfigMap
kubectl apply -f ./.local-dev/tilt_resources/postgres/postgres-pvc-pv.yaml
kubectl apply -f ./.local-dev/tilt_resources/postgres/postgres-config.yaml

# Step 2: Tyk ConfigMap
kubectl create configmap tyk-config \
  --from-file=tyk.conf=./.local-dev/tilt_resources/tyk/tyk.conf \
  --from-file=default-api.json=./.local-dev/tilt_resources/tyk/default-api.json \
  --from-file=checkout-api.json=./.local-dev/tilt_resources/tyk/checkout-api.json \
  --from-file=default-policy.json=./.local-dev/tilt_resources/tyk/default-policy.json

# Step 3: Kratos (ConfigMaps + Deployment + Service)
kubectl apply -f ./.local-dev/tilt_resources/kratos/basic_k8s.yaml
# If kratos-migration job already exists:
kubectl delete job kratos-migration 2>/dev/null
kubectl apply -f ./.local-dev/tilt_resources/kratos/basic_k8s.yaml

# Step 4: Port-forward Kratos if Tilt didn't pick it up
kubectl port-forward svc/kratos-service 4433:4433 4434:4434 &
```

---

## Phase 4: Diagnose CrashLoopBackOff Pods

For each pod in CrashLoopBackOff or Error, check logs:

```bash
kubectl logs <pod-name> --tail=30
```

### Common crash patterns and fixes:

#### 4a. Dirty database migration

**Log pattern:**
```
panic: Dirty database version XXXXXXXXXX. Fix and force version.
```

**Fix:**
```bash
kubectl port-forward svc/postgres 5433:5432 &

# Check migration state
psql "postgresql://<svc_user>:@localhost:5433/<db_name>" \
  -c "SELECT version, dirty FROM schema_migrations;"

# Clear dirty flag
psql "postgresql://<svc_user>:@localhost:5433/<db_name>" \
  -c "UPDATE schema_migrations SET dirty = false WHERE version = <version>;"

# Restart the pod
kubectl delete pod <pod-name>
```

#### 4b. Missing migration file

**Log pattern:**
```
panic: no migration found for version XXXXXXXXXX
```

**Fix:** The DB has a version that doesn't exist in the code. Find the latest migration that exists and reset:

```bash
# List available migrations
ls <service>/migrations/*.up.sql | sort | tail -5

# Set version to the last known good migration before the missing one
psql "postgresql://<svc_user>:@localhost:5433/<db_name>" \
  -c "UPDATE schema_migrations SET version = <last_good_version>, dirty = false;"

kubectl delete pod <pod-name>
```

#### 4c. Missing pg_trgm extension

**Log pattern:**
```
panic: Dirty database version 20260320182604
```
(Specifically the gin_trgm index migration for svc-accounts-payable)

**Fix:**
```bash
psql "postgresql://svc_accounts_payable:@localhost:5433/alt_accounts_payable" \
  -c "CREATE EXTENSION IF NOT EXISTS pg_trgm;"
psql "postgresql://svc_accounts_payable:@localhost:5433/alt_accounts_payable" \
  -c "UPDATE schema_migrations SET dirty = false;"

kubectl delete pod -l app=svc-accounts-payable
```

#### 4d. RabbitMQ service missing

**Log pattern:**
```
lookup alt-amqp on 10.96.0.10:53: no such host
```
Or services die silently after logging RabbitMQ defaults.

**Fix:**
```bash
# Check if service exists
kubectl get svc | grep amqp

# If missing, create it
kubectl expose pod alt-amqp --port=5672 --name=alt-amqp

# Restart affected pods
kubectl delete pod -l app=svc-accounts-payable
kubectl delete pod -l app=svc-payments-core
```

#### 4e. Missing database

**Log pattern:**
```
FATAL: database "alt_<service>" does not exist
```

**Fix:** Run the reset-db script or create the database manually:

```bash
psql "postgresql://postgresadmin:@localhost:5433/postgresdb" \
  -c "CREATE DATABASE alt_<service>;"
psql "postgresql://postgresadmin:@localhost:5433/postgresdb" \
  -c "CREATE ROLE svc_<service> LOGIN;"
psql "postgresql://postgresadmin:@localhost:5433/postgresdb" \
  -c "ALTER DATABASE alt_<service> OWNER TO svc_<service>;"
```

---

## Phase 5: AP Access Verification

If `svc-accounts-payable` was affected, run the AP access checklist:

```bash
# 1. Service running?
kubectl get pods | grep -E "svc-accounts-payable|svc-payments-core|svc-partner-gateway"

# 2. RabbitMQ accessible?
kubectl get svc | grep amqp

# 3. DB onboarding status correct?
kubectl port-forward svc/postgres 5433:5432 &

psql "postgresql://svc_accounts_payable:@localhost:5433/alt_accounts_payable" \
  -c "SELECT partner_id, onboarding_status FROM partner_configs WHERE partner_id = '2b36efd3-8952-4bc8-921b-5b4ce9011b40';"

psql "postgresql://svc_partner:@localhost:5433/alt_partner" \
  -c "SELECT id, adyen_onboarding_status FROM partners WHERE id = '2b36efd3-8952-4bc8-921b-5b4ce9011b40';"
```

If `onboarding_status` is not `concluded`:
```sql
UPDATE partner_configs SET onboarding_status = 'concluded'
WHERE partner_id = '2b36efd3-8952-4bc8-921b-5b4ce9011b40';
```

If `adyen_onboarding_status` is not `concluded`:
```sql
UPDATE partners SET adyen_onboarding_status = 'concluded'
WHERE id = '2b36efd3-8952-4bc8-921b-5b4ce9011b40';
```

---

## Phase 6: Verify

```bash
# Infrastructure pods
kubectl get pod -l app=postgres
kubectl get pod -l app=tyk
kubectl get pod -l app=kratos

# Kratos health
curl -s http://localhost:4433/health/ready
# Expected: {"status":"ok"}

# Gateway health
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"query":"{ __typename }"}' http://localhost:4001/query
# Expected: {"data":{"__typename":"Query"}}

# Final pod status
kubectl get pods | grep -E "CrashLoop|Error|Pending"
```

---

## Phase 7: Report

Present a summary to the user:

```markdown
## Service Update Complete

### Repositories
- Updated: X/Y
- Failed: [list if any]
- Stashed: [list if any]

### Pod Health
- Running: X
- Fixed: [list of pods that were fixed and what was wrong]
- Still failing: [list if any, with reason]

### Infrastructure
- Postgres: Running/Fixed/Issue
- Tyk: Running/Fixed/Issue
- Kratos: Running/Fixed/Issue
- RabbitMQ: Running/Fixed/Issue

### Actions Taken
- [list of fixes applied]

### Manual Action Required
- [list anything that needs user intervention, if any]
```

---

## Rules

1. **Always run update-all.sh first** - pull latest code before diagnosing
2. **Check pods after update** - new code may introduce new migrations that break
3. **Fix migrations before restarting** - clearing dirty flag without fixing the cause just delays the crash
4. **Don't force-push or reset repos** - the update script uses --ff-only for safety
5. **Report stashed repos** - user needs to know which repos had local changes stashed
6. **Port-forward cleanup** - kill any background port-forwards when done
7. **Kratos port-forward** - always ensure 4433:4433 and 4434:4434 are forwarded for frontend auth
