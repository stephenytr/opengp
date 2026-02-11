# OpenGP TUI - Quick Start Guide

## Running the Application

### Easiest Way: Use the run script

```bash
./run-dev.sh
```

### Manual Setup

```bash
export ENCRYPTION_KEY=0000000000000000000000000000000000000000000000000000000000000000
export DATABASE_URL=sqlite:opengp.db
cargo run
```

## Controls

| Key | Action |
|-----|--------|
| `1` | Patients screen |
| `2` | Appointments screen |
| `3` | Clinical screen |
| `4` | Billing screen |
| `Tab` | Next screen |
| `Shift+Tab` | Previous screen |
| `q` or `Ctrl+C` | Quit |

## What's Working

✅ Event loop and rendering  
✅ Navigation between screens  
✅ Keyboard shortcuts  
✅ Component architecture  

## What's Not Implemented Yet

⏳ Individual screen components (showing placeholders)  
⏳ Patient management  
⏳ Appointment scheduling  
⏳ Clinical records  
⏳ Billing functionality  

---

**See README-TUI.md for detailed documentation**
