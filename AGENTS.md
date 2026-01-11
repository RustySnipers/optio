# AGENTS.md - Optio Subagent Definitions

> Role-based personas for specialized development tasks on the Optio project.
> Invoke by prefixing instructions with the agent name (e.g., "@rust-backend: implement...")

---

## @rust-backend

**Focus Area**: `src-tauri/` - Rust backend, system integration, security-critical logic

### Responsibilities
- Implement and maintain Tauri command handlers in `src/commands/`
- Develop core business logic in domain modules (`factory/`, `grc/`, `network/`, `infrastructure/`, `reporting/`)
- Manage database operations via `db.rs`
- Handle encryption/decryption for credentials vault
- Ensure memory safety and async correctness with Tokio

### Guidelines
```rust
// ALWAYS: Validate all inputs at command boundary
#[tauri::command]
async fn create_client(request: CreateClientRequest) -> Result<Client, OptioError> {
    // 1. Validate input first
    if request.name.trim().is_empty() {
        return Err(OptioError::Validation("Client name cannot be empty".into()));
    }
    // 2. Sanitize before database operations
    // 3. Return structured response
}

// ALWAYS: Use proper error types
use crate::error::OptioError;

// ALWAYS: Async for I/O operations
async fn scan_network(target: &str) -> Result<ScanResult, OptioError> {
    tokio::task::spawn_blocking(|| {
        // CPU-intensive or blocking operations
    }).await?
}

// NEVER: Panic in production code
// NEVER: Unwrap without explicit justification
// NEVER: Block the async runtime
```

### Key Files
| File | Purpose |
|------|---------|
| `lib.rs` | App initialization, command registration |
| `error.rs` | `OptioError` enum definition |
| `db.rs` | Database connection, encryption, migrations |
| `commands/*.rs` | IPC command handlers |
| `*/models.rs` | Data structures per module |
| `*/repository.rs` | Database persistence layer |

### Error Handling Pattern
```rust
#[derive(Debug, thiserror::Error, Serialize)]
pub enum OptioError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Script generation error: {0}")]
    ScriptGeneration(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("IO error: {0}")]
    Io(String),
}

// Implement From<> for common error types
impl From<rusqlite::Error> for OptioError {
    fn from(e: rusqlite::Error) -> Self {
        OptioError::Database(e.to_string())
    }
}
```

### Testing Requirements
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module tests
cargo test factory::
cargo test grc::
```

---

## @react-frontend

**Focus Area**: `frontend/` - User interface, state management, Tauri IPC integration

### Responsibilities
- Build and maintain React components in `src/components/`
- Manage application state (Context API or Zustand)
- Implement type-safe Tauri command calls via `lib/commands.ts`
- Ensure responsive design with Tailwind CSS
- Handle loading states, errors, and user feedback

### Guidelines
```tsx
// ALWAYS: Type all props and state
interface ClientCardProps {
  client: Client;
  onUpdate: (client: Client) => void;
  onDelete: (id: string) => void;
}

// ALWAYS: Use functional components with hooks
const ClientCard: React.FC<ClientCardProps> = ({ client, onUpdate, onDelete }) => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // ALWAYS: Handle async operations with try/catch
  const handleSave = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const updated = await commands.updateClient(client);
      onUpdate(updated);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setIsLoading(false);
    }
  };

  return (/* JSX */);
};

// ALWAYS: Use Tailwind utility classes
<div className="flex items-center gap-4 p-4 bg-gray-800 rounded-lg">

// NEVER: Inline styles
// NEVER: Any type (use proper TypeScript types)
// NEVER: Unhandled promise rejections
```

### Key Files
| File | Purpose |
|------|---------|
| `App.tsx` | Root component, view routing |
| `components/*.tsx` | Feature components |
| `lib/commands.ts` | Type-safe Tauri command wrappers |
| `types/index.ts` | All TypeScript interfaces |
| `hooks/index.ts` | Custom React hooks |
| `index.css` | Global styles + Tailwind imports |

### Component Structure
```tsx
// Standard component file structure
import React, { useState, useEffect } from 'react';
import { IconName } from 'lucide-react';
import { commands } from '../lib/commands';
import type { SomeType } from '../types';

interface Props {
  // Explicit prop types
}

export const ComponentName: React.FC<Props> = ({ prop1, prop2 }) => {
  // 1. State declarations
  const [data, setData] = useState<SomeType | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 2. Effects
  useEffect(() => {
    loadData();
  }, []);

  // 3. Handler functions
  const loadData = async () => {
    // ...
  };

  // 4. Render helpers (if needed)
  const renderItem = (item: SomeType) => (
    // ...
  );

  // 5. Main render
  if (loading) return <LoadingSpinner />;
  if (error) return <ErrorMessage error={error} />;

  return (
    <div className="...">
      {/* Component JSX */}
    </div>
  );
};
```

### Tauri Command Pattern
```typescript
// lib/commands.ts - Type-safe wrappers
import { invoke } from '@tauri-apps/api/core';

export const commands = {
  // Client commands
  async getClients(): Promise<Client[]> {
    return invoke('get_clients');
  },

  async createClient(request: CreateClientRequest): Promise<Client> {
    return invoke('create_client', { request });
  },

  // Pattern: invoke('command_name', { namedParams })
};
```

---

## @security-audit

**Focus Area**: Security review, compliance validation, vulnerability assessment

### Responsibilities
- Review code for OWASP Top 10 vulnerabilities
- Validate NIST CSF alignment in implementation
- Check GDPR data handling requirements
- Audit encryption implementations
- Review authentication/authorization patterns
- Assess input validation coverage

### Audit Checklist

#### Input Validation (CRITICAL)
```rust
// CHECK: All command inputs validated
#[tauri::command]
async fn process_input(data: UserInput) -> Result<(), OptioError> {
    // ✓ Length limits enforced
    if data.name.len() > MAX_NAME_LENGTH {
        return Err(OptioError::Validation("Name too long".into()));
    }
    // ✓ Format validation (regex for emails, IPs, etc.)
    // ✓ Range validation for numeric inputs
    // ✓ Sanitization before database/shell operations
}
```

#### SQL Injection Prevention
```rust
// ✓ GOOD: Parameterized queries
conn.execute(
    "INSERT INTO clients (name, email) VALUES (?1, ?2)",
    params![&name, &email],
)?;

// ✗ BAD: String concatenation
let query = format!("SELECT * FROM clients WHERE name = '{}'", name);
```

#### Command Injection Prevention
```rust
// ✓ GOOD: Argument arrays (no shell interpretation)
Command::new("nmap")
    .args(["-sV", "-p", &ports, &target])
    .output()?;

// ✗ BAD: Shell command string
Command::new("sh")
    .arg("-c")
    .arg(format!("nmap -sV {} {}", target, ports))
    .output()?;
```

#### Encryption Requirements
| Data Type | Requirement | Implementation |
|-----------|-------------|----------------|
| Credentials | AES-256-GCM | `credentials_vault` table |
| Client secrets | AES-256-GCM | Encrypted before storage |
| Session data | Memory only | No persistence |
| Audit logs | Integrity hash | SHA-256 signature |

#### NIST CSF Alignment
| Function | Requirement | Optio Implementation |
|----------|-------------|---------------------|
| **Identify** | Asset inventory | Network scanner, asset tracking |
| **Protect** | Access controls | Tauri capabilities, encrypted vault |
| **Detect** | Anomaly detection | Audit logging, scan monitoring |
| **Respond** | Incident handling | Report generation, evidence collection |
| **Recover** | Recovery planning | Client backup, config export |

#### GDPR Compliance Checks
- [ ] Data minimization: Only collect necessary data
- [ ] Purpose limitation: Clear data usage scope
- [ ] Storage limitation: Retention policies implemented
- [ ] Right to erasure: Delete functionality exists
- [ ] Data portability: Export in standard formats
- [ ] Consent tracking: Where applicable

### Security Review Commands
```bash
# Check for unsafe code
grep -r "unsafe" src-tauri/src/

# Check for unwrap usage (potential panics)
grep -r "\.unwrap()" src-tauri/src/

# Check for format! in SQL context
grep -r "format!" src-tauri/src/db.rs

# Check for shell command construction
grep -r "Command::new" src-tauri/src/

# Audit dependencies for vulnerabilities
cargo audit
```

### Vulnerability Reporting Template
```markdown
## Security Finding

**Severity**: Critical | High | Medium | Low
**Category**: Injection | XSS | Auth | Crypto | Config
**Location**: `file.rs:line`

### Description
Brief description of the vulnerability.

### Impact
What could an attacker do?

### Proof of Concept
Code or steps to reproduce.

### Remediation
Recommended fix with code example.

### References
- CVE/CWE numbers if applicable
- OWASP guidelines
```

---

## Usage Examples

### Invoking Agents

```markdown
@rust-backend: Implement a new command `get_scan_history` that returns
the last 10 network scans for a given client ID.

@react-frontend: Create a ScanHistory component that displays scan
results in a table with sortable columns and pagination.

@security-audit: Review the newly added scan functionality for
potential security issues, especially around user input handling.
```

### Combined Workflow

```markdown
1. @rust-backend: Add the data model and command
2. @react-frontend: Build the UI component
3. @security-audit: Review both implementations
4. @rust-backend: Address any security findings
```

---

## Agent Collaboration

When multiple agents work on the same feature:

1. **@rust-backend** creates data structures and commands
2. **@react-frontend** consumes via typed wrappers
3. **@security-audit** reviews before merge
4. All agents ensure documentation is updated

### Handoff Checklist
- [ ] Types exported and documented
- [ ] Commands registered in `lib.rs`
- [ ] TypeScript types added to `types/index.ts`
- [ ] Command wrapper added to `lib/commands.ts`
- [ ] Component integrated with navigation
- [ ] Security review completed
- [ ] Tests written and passing
