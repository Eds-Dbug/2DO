# Todo Calendar - iCalendar Files

This directory contains your iCalendar (.ics) files that the Todo Calendar app will read from.

## How to Use

1. **Add Calendar Files**: Place your `.ics` files in this directory
2. **File Format**: Files must be valid iCalendar format with `.ics` extension
3. **Portable**: This entire folder travels with the app on USB drives

## Sample Files

- `sample-todos.ics` - Example calendar with sample tasks

## Creating iCalendar Files

You can create `.ics` files from:
- Google Calendar (Export as .ics)
- Microsoft Outlook (Export as .ics)
- Apple Calendar (Export as .ics)
- Any calendar app that supports iCalendar format

## File Structure

Each `.ics` file should contain VTODO components for tasks:

```
BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VTODO
UID:unique-id@example.com
SUMMARY:Task Title
DESCRIPTION:Task Description
DUE;VALUE=DATE:20241215
PRIORITY:1
STATUS:NEEDS-ACTION
CATEGORIES:work
END:VTODO
END:VCALENDAR
```

## Priority Levels

- 1-3: High priority
- 4-6: Medium priority  
- 7-9: Low priority

## Status Values

- `NEEDS-ACTION`: Incomplete task
- `COMPLETED`: Completed task
