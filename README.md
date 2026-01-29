# Cutie

> A calm, personal task management app designed for Perceivers (P-types in MBTI)

Cutie is a desktop application that reimagines task management through the lens of "Life as Theater" - where you are the director arranging your tasks like actors on a stage. Built for individuals who value flexibility over rigid structure, Cutie embraces change rather than punishing it.

## Philosophy

### Core Principles

1. **Minimize Meta-Work** - Reduce the work of managing the tool itself
2. **Process Over Outcome** - Acknowledge effort, not just completion
3. **User Autonomy** - AI assists but never decides; you're always the director
4. **Personal Space is Sacred** - No team features; this is your private sanctuary
5. **Embrace Change** - Plan changes are normal; overdue tasks aren't failures

### What Makes Cutie Different

| Traditional Tools | Cutie |
|------------------|-------|
| Backlog = "accumulated debt" (negative) | Staging = "resources awaiting dispatch" (neutral) |
| Red overdue labels (guilt) | Tasks flow back to Staging (acceptance) |
| Complete or fail | **Presence** - record your effort even if unfinished |
| Manual tagging | AI auto-assigns Areas (zero meta-work) |
| API integrations | VLM screenshot import (capture anything) |

## Features

### Staging System
Tasks without a scheduled date live in Staging - a neutral holding area that replaces the anxiety-inducing "backlog". When plans change, tasks simply return here without judgment.

### Presence Mechanism
Did you spend 2 hours on a task but didn't finish? Click **Presence** to:
- Lock the time block on your calendar (your effort is recorded)
- Return the task to Staging for rescheduling
- No guilt, no red labels - just acknowledgment of your work

### Time Blocks (Many-to-Many)
Unlike traditional tools where one task = one time slot:
- A time block can contain multiple tasks (theme-based planning)
- A task can span multiple time blocks (segmented execution)
- Time blocks cannot overlap (realistic constraint)

### AI-Powered Features
- **Auto Area Assignment** - Tasks are automatically categorized and color-coded
- **Staging Organization** - AI suggests project groupings
- **Subtask Generation** - Gentle, encouraging micro-tasks to help you start
- **VLM Screenshot Import** - Capture tasks from any screen (emails, chats, handwritten notes)

### Templates & Rituals
Create personal rituals and routines:
- Morning startup sequences
- Custom shutdown ceremonies
- "Small happiness" rituals (tea time, walks, etc.)

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri
- **Database**: SQLite
- **State Management**: Pinia
- **Architecture**: CPU-inspired pipeline system with 5-stage execution (IF-SCH-EX-RES-WB)

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)

### Installation

```bash
# Clone the repository
git clone https://github.com/anthropics/cutie.git
cd cutie

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Project Structure

```
cutie/
├── src/                    # Frontend (Vue 3)
│   ├── components/         # UI components (atomic design)
│   ├── composables/        # Business logic hooks
│   ├── cpu/                # CPU pipeline system
│   ├── stores/             # Pinia stores (RTL hardware design)
│   └── views/              # Page components
├── src-tauri/              # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── entities/       # Domain models & DTOs
│   │   ├── features/       # Feature modules (vertical slices)
│   │   └── shared/         # Cross-cutting concerns
│   └── migrations/         # SQLite migrations
└── references/             # Design documents
```

## Contributing

We welcome contributions! Please read our contributing guidelines before submitting PRs.

### Development Commands

```bash
# Frontend type checking
pnpm exec vue-tsc --noEmit

# Lint
pnpm exec eslint src/

# Format
pnpm exec prettier --check .

# Backend checks
cargo check
cargo clippy
cargo test
```

## Roadmap

### V1.0 - Core (Current)
- [x] Task management system
- [x] Staging and Stage system
- [x] Time block management
- [x] Cross-view drag & drop
- [x] Presence mechanism
- [ ] AI system integration
- [ ] VLM screenshot recognition
- [ ] Template system

### V1.5 - Enhanced
- [ ] Advanced AI features
- [ ] Custom ritual system
- [ ] Data import/export
- [ ] Themes and personalization

### V2.0 - Project Management
- [ ] Bi-directional linked notes
- [ ] Block editor
- [ ] Advanced search and filtering

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Cutie is inspired by the belief that productivity tools should adapt to humans, not the other way around. Special thanks to all P-types who've struggled with rigid task managers - this one's for you.

---

*"On life's stage, everyone is their own director. Cutie just helps you arrange when the actors appear."*
