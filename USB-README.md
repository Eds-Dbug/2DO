# Todo Calendar - USB Portable Version

A portable todo/calendar application that reads from iCalendar files.

## Quick Start

1. **Run the app**: Double-click `d0.exe` (or the appropriate executable for your OS)
2. **Add calendars**: Place `.ics` files in the `calendars/` folder
3. **Select calendar**: Choose a calendar from the list to view its tasks
4. **Manage tasks**: Switch between list view and calendar view

## File Structure

```
Todo Calendar/
├── d0.exe                    # Main application
├── calendars/                # Your iCalendar files go here
│   ├── sample-todos.ics     # Example calendar
│   └── README.md            # Instructions for calendar files
└── USB-README.md            # This file
```

## Features

- **Portable**: Runs from USB drive, no installation required
- **iCalendar Support**: Works with any calendar app (Google, Outlook, Apple, etc.)
- **Multiple Calendars**: Switch between different calendar files
- **Two Views**: List view and calendar view
- **Task Management**: View, complete, and organize tasks
- **Priority Support**: High, medium, low priority tasks
- **Categories**: Organize tasks by category

## Adding Calendar Files

1. Export your calendar as `.ics` from your calendar app
2. Copy the `.ics` file to the `calendars/` folder
3. Restart the app or click "Refresh" to see the new calendar

## Supported Calendar Apps

- Google Calendar
- Microsoft Outlook
- Apple Calendar
- Thunderbird
- Any app that exports iCalendar format

## System Requirements

- Windows 10/11 (or macOS/Linux with appropriate executable)
- No additional software required

## Troubleshooting

- **No calendars found**: Make sure `.ics` files are in the `calendars/` folder
- **App won't start**: Check that all files are present and not corrupted
- **Tasks not showing**: Verify the `.ics` file contains VTODO components

## Data Storage

All data is stored locally in the `calendars/` folder. This makes the app completely portable - just copy the entire folder to any USB drive or computer.
