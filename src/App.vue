<script setup>
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Reactive state
const calendars = ref([])
const todos = ref([])
const selectedCalendar = ref(null)
const currentView = ref('list')
const showTaskForm = ref(false)
const isEditingTask = ref(false) // Track if we're editing an existing task
const editingTaskId = ref(null) // ID of the task being edited
const selectedDate = ref(null)
const currentDate = ref(new Date())
const loading = ref(false)
const saving = ref(false) // New: show saving state
const showCalendarSelection = ref(true) // New: show calendar selection first
const calendarsPath = ref('')
const calendarUpdateKey = ref(0) // Force calendar re-render
const showCompletedInCalendar = ref(true) // Show completed tasks in calendar
const showCompletedInList = ref(true) // Show completed tasks in list view
const searchQuery = ref('') // Search filter for tasks
const incompleteSortOrder = ref('desc') // Sort order for incomplete tasks: 'asc' or 'desc'
const completedSortOrder = ref('desc') // Sort order for completed tasks: 'asc' or 'desc'

// Calendar sidebar state
const sidebarTasks = ref([]) // Tasks for the selected date
const showSidebar = ref(false) // Whether to show the sidebar

// New task form data
const newTask = ref({
  title: '',
  description: '',
  priority: 'medium',
  category: '',
  dueDate: ''
})

// Load calendars on startup
onMounted(async () => {
  await loadCalendars()
  await loadCalendarsPath()
})

// Watch for changes in todos
watch(todos, () => {
  calendarUpdateKey.value++ // Force calendar re-render
}, { deep: true })

// Watch for changes in showCompletedInCalendar
watch(showCompletedInCalendar, () => {
  calendarUpdateKey.value++ // Force calendar re-render
})


// Load all available calendars
const loadCalendars = async () => {
  try {
    loading.value = true
    calendars.value = await invoke('list_calendars')
  } catch (error) {
    console.error('Failed to load calendars:', error)
  } finally {
    loading.value = false
  }
}

// Load todos from selected calendar
const loadTodosFromCalendar = async (calendar) => {
  try {
    loading.value = true
    selectedCalendar.value = calendar
    const loadedTodos = await invoke('load_todos_from_calendar', { calendarPath: calendar.path })
    todos.value = loadedTodos
    showCalendarSelection.value = false // Hide calendar selection, show todo app
  } catch (error) {
    console.error('Failed to load todos:', error)
    alert('Failed to load todos from calendar. Please try again.')
  } finally {
    loading.value = false
  }
}

// Go back to calendar selection
const goBackToCalendarSelection = () => {
  showCalendarSelection.value = true
  selectedCalendar.value = null
  todos.value = []
}

// Load calendars directory path
const loadCalendarsPath = async () => {
  try {
    calendarsPath.value = await invoke('get_calendars_path')
    console.log('Calendars path loaded:', calendarsPath.value)
  } catch (error) {
    console.error('Failed to load calendars path:', error)
    // Show the actual error instead of a fallback
    calendarsPath.value = `Error: ${error}`
  }
}

// Helper function to sort todos by a given sort order
const sortTodosByOrder = (todos, sortOrder) => {
  return [...todos].sort((a, b) => {
    // Primary sort: by due date (respecting sort order)
    if (a.dueDate && b.dueDate) {
      const dateA = toDate(a.dueDate)
      const dateB = toDate(b.dueDate)
      const dateDiff = sortOrder === 'asc' ? dateA - dateB : dateB - dateA
      if (dateDiff !== 0) {
        return dateDiff
      }
    } else if (a.dueDate && !b.dueDate) {
      return sortOrder === 'asc' ? -1 : 1 // Tasks with due dates come first in desc, last in asc
    } else if (!a.dueDate && b.dueDate) {
      return sortOrder === 'asc' ? 1 : -1
    }
    
    // Secondary sort: by creation date (if available)
    if (a.createdAt && b.createdAt) {
      const dateA = toDate(a.createdAt)
      const dateB = toDate(b.createdAt)
      const dateDiff = sortOrder === 'asc' ? dateA - dateB : dateB - dateA
      if (dateDiff !== 0) {
        return dateDiff
      }
    } else if (a.createdAt && !b.createdAt) {
      return sortOrder === 'asc' ? -1 : 1
    } else if (!a.createdAt && b.createdAt) {
      return sortOrder === 'asc' ? 1 : -1
    }
    
    // Tertiary sort: by priority
    const priorityOrder = { high: 3, medium: 2, low: 1 }
    const priorityDiff = priorityOrder[b.priority] - priorityOrder[a.priority]
    if (priorityDiff !== 0) {
      return priorityDiff
    }
    
    // Quaternary sort: by title (alphabetical, respecting sort order)
    const titleA = a.title.toLowerCase()
    const titleB = b.title.toLowerCase()
    const titleDiff = titleA.localeCompare(titleB)
    if (titleDiff !== 0) {
      return sortOrder === 'asc' ? titleDiff : -titleDiff
    }
    
    // Quinary sort: by ID (for consistent ordering)
    const idDiff = a.id - b.id
    return sortOrder === 'asc' ? idDiff : -idDiff
  })
}

// Filter todos based on search query
const filteredTodos = computed(() => {
  if (!searchQuery.value.trim()) {
    return todos.value
  }
  
  const query = searchQuery.value.toLowerCase().trim()
  return todos.value.filter(todo => {
    const title = todo.title.toLowerCase()
    const description = (todo.description || '').toLowerCase()
    const category = (todo.category || '').toLowerCase()
    
    return title.includes(query) || 
           description.includes(query) || 
           category.includes(query)
  })
})

// Separate completed and incomplete tasks for list view with individual sorting
const incompleteTodos = computed(() => {
  const incomplete = filteredTodos.value.filter(todo => !todo.completed)
  return sortTodosByOrder(incomplete, incompleteSortOrder.value)
})

const completedTodos = computed(() => {
  const completed = filteredTodos.value.filter(todo => todo.completed)
  return sortTodosByOrder(completed, completedSortOrder.value)
})

const currentMonthYear = computed(() => {
  return currentDate.value.toLocaleDateString('en-US', { 
    month: 'long', 
    year: 'numeric' 
  })
})

const dayHeaders = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']

const calendarDays = computed(() => {
  const year = currentDate.value.getFullYear()
  const month = currentDate.value.getMonth()
  
  const firstDay = new Date(year, month, 1)
  const lastDay = new Date(year, month + 1, 0)
  const startDate = new Date(firstDay)
  startDate.setDate(startDate.getDate() - firstDay.getDay())
  
  const days = []
  const currentDateObj = new Date(startDate)
  
  for (let i = 0; i < 42; i++) {
    days.push({
      date: new Date(currentDateObj),
      isCurrentMonth: currentDateObj.getMonth() === month
    })
    currentDateObj.setDate(currentDateObj.getDate() + 1)
  }
  
  return days
})

// Methods
const toggleTodo = async (id) => {
  const todo = todos.value.find(t => t.id === id)
  if (todo) {
    todo.completed = !todo.completed
    // Save changes to file
    await saveTodosToFile()
  }
}

const deleteTodo = async (id) => {
  todos.value = todos.value.filter(t => t.id !== id)
  // Save changes to file
  await saveTodosToFile()
}

const editTodo = (id) => {
  const todo = todos.value.find(t => t.id === id)
  if (todo) {
    console.log('Editing todo:', todo)
    isEditingTask.value = true
    editingTaskId.value = id
    
    // Format due date using local date to avoid timezone issues
    const formattedDueDate = formatDateForInput(todo.dueDate)
    
    newTask.value = {
      title: todo.title,
      description: todo.description || '',
      priority: todo.priority,
      category: todo.category || '',
      dueDate: formattedDueDate
    }
    console.log('Form populated with:', newTask.value)
    showTaskForm.value = true
  }
}

// Save todos back to the calendar file
const saveTodosToFile = async () => {
  if (!selectedCalendar.value) {
    console.error('No calendar selected, cannot save')
    return
  }
  
  try {
    saving.value = true
    console.log('Saving todos to file:', selectedCalendar.value.path)
    console.log('Todos to save:', todos.value.length, 'items')
    
    // Ensure todos have correct format for backend compatibility
    const todosWithCalendarName = todos.value.map(todo => ({
      ...todo,
      id: todo.id.toString(), // Ensure ID is always a string
      calendar_name: selectedCalendar.value.name,
      // Ensure dates are strings
      dueDate: todo.dueDate ? (typeof todo.dueDate === 'string' ? todo.dueDate : todo.dueDate.toISOString().split('T')[0]) : null,
      createdAt: todo.createdAt ? (typeof todo.createdAt === 'string' ? todo.createdAt : todo.createdAt.toISOString()) : null
    }))
    
    await invoke('save_todos_to_calendar', { 
      calendarPath: selectedCalendar.value.path, 
      todos: todosWithCalendarName 
    })
    console.log('Todos saved successfully to file')
  } catch (error) {
    console.error('Failed to save todos:', error)
    alert('Failed to save changes. Please try again.')
  } finally {
    saving.value = false
  }
}

const createTask = async () => {
  console.log('createTask called, isEditingTask:', isEditingTask.value)
  console.log('Form data:', newTask.value)
  
  if (!newTask.value.title.trim()) {
    console.log('No title provided, returning')
    return
  }
  
  if (isEditingTask.value) {
    // Update existing task
    const taskIndex = todos.value.findIndex(t => t.id === editingTaskId.value)
    console.log('Updating task at index:', taskIndex, 'with ID:', editingTaskId.value)
    console.log('All todo IDs:', todos.value.map(t => t.id))
    
    if (taskIndex !== -1) {
      const existingTask = todos.value[taskIndex]
      console.log('Original task:', existingTask)
      todos.value[taskIndex] = {
        ...existingTask,
        title: newTask.value.title,
        description: newTask.value.description,
        priority: newTask.value.priority,
        category: newTask.value.category || null,
        dueDate: newTask.value.dueDate ? createLocalDate(newTask.value.dueDate) : null,
      }
      console.log('Updated task:', todos.value[taskIndex])
      console.log('Task updated successfully')
    } else {
      console.error('Task not found for ID:', editingTaskId.value)
    }
  } else {
    // Create new task
    const task = {
      id: Date.now().toString(),
      title: newTask.value.title,
      description: newTask.value.description,
      completed: false,
      priority: newTask.value.priority,
      category: newTask.value.category || null,
      dueDate: newTask.value.dueDate ? createLocalDate(newTask.value.dueDate) : null,
      createdAt: new Date()
    }
    
    todos.value.push(task)
  }
  
  closeTaskForm()
  
  // Save changes to file
  await saveTodosToFile()
}

const closeTaskForm = () => {
  showTaskForm.value = false
  isEditingTask.value = false
  editingTaskId.value = null
  selectedDate.value = null
  newTask.value = {
    title: '',
    description: '',
    priority: 'medium',
    category: '',
    dueDate: ''
  }
}

const selectDate = (date) => {
  selectedDate.value = date
  sidebarTasks.value = getTasksForDate(date)
  showSidebar.value = true
}

const previousMonth = () => {
  currentDate.value = new Date(currentDate.value.getFullYear(), currentDate.value.getMonth() - 1, 1)
}

const nextMonth = () => {
  currentDate.value = new Date(currentDate.value.getFullYear(), currentDate.value.getMonth() + 1, 1)
}

const isToday = (date) => {
  const today = new Date()
  return date.toDateString() === today.toDateString()
}

const getTasksForDate = (date) => {
  return todos.value.filter(todo => {
    if (!todo.dueDate) return false
    
    // Parse date string in local time to avoid timezone issues
    const todoDate = typeof todo.dueDate === 'string' 
      ? new Date(todo.dueDate + 'T00:00:00') // Force local time interpretation
      : todo.dueDate
    
    const isOnDate = todoDate.toDateString() === date.toDateString()
    
    // If not showing completed tasks, filter them out
    if (!showCompletedInCalendar.value && todo.completed) {
      return false
    }
    
    return isOnDate
  })
}

const getPriorityColor = (priority) => {
  const colors = {
    high: 'bg-red-100 text-red-800 border-red-200',
    medium: 'bg-yellow-100 text-yellow-800 border-yellow-200',
    low: 'bg-green-100 text-green-800 border-green-200'
  }
  return colors[priority] || colors.medium
}

const formatDate = (date) => {
  const dateObj = toDate(date)
  return dateObj.toLocaleDateString('en-US', { 
    month: 'short', 
    day: 'numeric',
    year: 'numeric'
  })
}

// Helper function to format date for input field (YYYY-MM-DD)
const formatDateForInput = (date) => {
  if (!date) return ''
  const dateObj = toDate(date)
  const year = dateObj.getFullYear()
  const month = String(dateObj.getMonth() + 1).padStart(2, '0')
  const day = String(dateObj.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

// Helper function to create a local date from YYYY-MM-DD string
const createLocalDate = (dateString) => {
  if (!dateString) return null
  const [year, month, day] = dateString.split('-').map(Number)
  return new Date(year, month - 1, day) // month is 0-indexed
}

// Parse dates flexibly from either a Date, a date-only string (YYYY-MM-DD),
// or an ISO datetime string (YYYY-MM-DDTHH:MM:SS[.sss][Z])
const toDate = (value) => {
  if (!value) return null
  if (value instanceof Date) return value
  if (typeof value === 'string') {
    const trimmed = value.trim()
    const hasT = trimmed.includes('T')
    return new Date(hasT ? trimmed : `${trimmed}T00:00:00`)
  }
  return new Date(value)
}

const formatLastModified = (timestamp) => {
  const date = new Date(parseInt(timestamp) * 1000)
  const now = new Date()
  const diffInHours = Math.floor((now - date) / (1000 * 60 * 60))
  
  if (diffInHours < 1) return 'just now'
  if (diffInHours < 24) return `${diffInHours}h ago`
  if (diffInHours < 48) return 'yesterday'
  return date.toLocaleDateString()
}

// Helper functions for search and sort
const clearSearch = () => {
  searchQuery.value = ''
}

const toggleIncompleteSortOrder = () => {
  incompleteSortOrder.value = incompleteSortOrder.value === 'asc' ? 'desc' : 'asc'
}

const toggleCompletedSortOrder = () => {
  completedSortOrder.value = completedSortOrder.value === 'asc' ? 'desc' : 'asc'
}

// Calendar sidebar functions

const closeSidebar = () => {
  showSidebar.value = false
  selectedDate.value = null
  sidebarTasks.value = []
}

const formatDateForSidebar = (date) => {
  return date.toLocaleDateString('en-US', { 
    weekday: 'long', 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
  })
}
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-slate-50 to-emerald-50 p-4">
    <div class="max-w-6xl mx-auto">
      <!-- Calendar Selection Screen -->
      <div v-if="showCalendarSelection">
        <!-- Header -->
        <header class="text-center mb-8">
          <h1 class="text-4xl font-bold text-slate-800 mb-2">Todo Calendar</h1>
          <p class="text-slate-600">Select a calendar to view your tasks</p>
        </header>

        <!-- Loading State -->
        <div v-if="loading" class="flex justify-center items-center py-12">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-emerald-500"></div>
        </div>

        <!-- Calendar List -->
        <div v-else-if="calendars.length === 0" class="text-center py-12">
          <div class="text-slate-400 text-lg mb-4">No calendar files found</div>
          <p class="text-slate-500 mb-6">Place .ics files in your calendars directory to get started</p>
          <div class="text-sm text-slate-400 bg-slate-100 p-4 rounded-lg max-w-md mx-auto">
            <p class="font-medium mb-2">Calendar files should be placed in:</p>
            <code class="text-xs">{{ calendarsPath || 'calendars/' }}</code>
          </div>
        </div>
        
        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div 
            v-for="calendar in calendars" 
            :key="calendar.path"
            @click="loadTodosFromCalendar(calendar)"
            class="bg-white rounded-lg shadow-sm border border-slate-200 p-6 hover:shadow-md transition-shadow cursor-pointer group"
          >
            <div class="flex items-start justify-between mb-4">
              <h3 class="text-lg font-semibold text-slate-800 group-hover:text-emerald-600 transition-colors">
                {{ calendar.name }}
              </h3>
              <svg class="w-5 h-5 text-slate-400 group-hover:text-emerald-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
            </div>
            
            <div class="space-y-2">
              <div class="flex items-center text-sm text-slate-600">
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                </svg>
                {{ calendar.todo_count }} tasks
              </div>
              
              <div class="flex items-center text-sm text-slate-500">
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Modified {{ formatLastModified(calendar.last_modified) }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Todo App Interface (existing) -->
      <div v-else>
        <!-- Header -->
        <header class="text-center mb-8">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-4">
              <h1 class="text-4xl font-bold text-slate-800">Todo Calendar</h1>
              <p class="text-slate-600">{{ selectedCalendar?.name }}</p>
              <div v-if="saving" class="flex items-center text-sm text-emerald-600">
                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-emerald-500 mr-2"></div>
                Saving...
              </div>
            </div>
            <button 
              @click="goBackToCalendarSelection"
              class="px-4 py-2 text-slate-600 hover:text-slate-800 hover:bg-slate-100 rounded-lg transition-colors"
            >
              ← Switch Calendar
            </button>
          </div>
          <p class="text-slate-600">Organize your tasks with calendar integration</p>
        </header>

      <!-- View Toggle -->
      <div class="flex justify-center mb-6">
        <div class="bg-white rounded-lg p-1 shadow-sm border">
          <button 
            @click="currentView = 'list'"
            :class="[
              'px-4 py-2 rounded-md transition-all duration-200',
              currentView === 'list' 
                ? 'bg-emerald-500 text-white shadow-sm' 
                : 'text-slate-600 hover:text-slate-800'
            ]"
          >
            List View
          </button>
          <button 
            @click="currentView = 'calendar'"
            :class="[
              'px-4 py-2 rounded-md transition-all duration-200',
              currentView === 'calendar' 
                ? 'bg-emerald-500 text-white shadow-sm' 
                : 'text-slate-600 hover:text-slate-800'
            ]"
          >
            Calendar View
          </button>
        </div>
      </div>

      <!-- List View -->
      <div v-if="currentView === 'list'" class="space-y-4">
        <!-- List View Controls -->
        <div class="space-y-4">
          <!-- Header with counts and completed toggle -->
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
              <h2 class="text-lg font-semibold text-slate-800">Tasks</h2>
              <div class="flex items-center gap-2 text-sm text-slate-600">
                <span>{{ incompleteTodos.length }} incomplete</span>
                <span v-if="showCompletedInList">• {{ completedTodos.length }} completed</span>
                <span v-if="searchQuery">• {{ filteredTodos.length }} filtered</span>
              </div>
            </div>
            <label class="flex items-center gap-2 text-sm">
              <input 
                type="checkbox" 
                v-model="showCompletedInList"
                class="rounded"
              >
              Show completed tasks
            </label>
          </div>

          <!-- Search Bar -->
          <div class="relative">
            <div class="relative">
              <input 
                v-model="searchQuery"
                type="text" 
                placeholder="Search tasks by title, description, or category..."
                class="w-full pl-10 pr-10 py-2 border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500"
              >
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="h-4 w-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
              <button 
                v-if="searchQuery"
                @click="clearSearch"
                class="absolute inset-y-0 right-0 pr-3 flex items-center text-slate-400 hover:text-slate-600"
              >
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
        </div>

        <div v-if="todos.length === 0" class="text-center py-12">
          <div class="text-slate-400 text-lg">No tasks yet</div>
          <p class="text-slate-500 mt-2">Click the + button to create your first task</p>
        </div>

        <!-- No search results -->
        <div v-else-if="searchQuery && filteredTodos.length === 0" class="text-center py-12">
          <div class="text-slate-400 text-lg">No tasks found</div>
          <p class="text-slate-500 mt-2">Try adjusting your search terms or clearing the search</p>
          <button 
            @click="clearSearch"
            class="mt-3 px-4 py-2 text-sm text-emerald-600 hover:text-emerald-700 hover:bg-emerald-50 rounded-lg transition-colors"
          >
            Clear Search
          </button>
        </div>
        
        <!-- Incomplete Tasks -->
        <div v-if="incompleteTodos.length > 0" class="space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-medium text-slate-600 uppercase tracking-wide">Incomplete Tasks</h3>
            <button 
              @click="toggleIncompleteSortOrder"
              class="flex items-center gap-2 px-4 py-2 text-sm bg-blue-100 text-blue-700 hover:bg-blue-200 rounded-lg transition-colors cursor-pointer border border-blue-200"
              :title="`Sort ${incompleteSortOrder === 'asc' ? 'ascending' : 'descending'}`"
            >
              <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
              </svg>
              <span>{{ incompleteSortOrder === 'asc' ? 'Oldest First' : 'Newest First' }}</span>
            </button>
          </div>
          <div v-for="todo in incompleteTodos" :key="todo.id" 
               class="bg-white rounded-lg shadow-sm border border-slate-200 p-4 hover:shadow-md transition-shadow">
            <div class="flex items-start justify-between">
            <div class="flex items-start space-x-3 flex-1">
              <button 
                @click="toggleTodo(todo.id)"
                :class="[
                  'mt-1 w-5 h-5 rounded border-2 flex items-center justify-center transition-colors',
                  todo.completed 
                    ? 'bg-emerald-500 border-emerald-500 text-white' 
                    : 'border-slate-300 hover:border-emerald-400'
                ]"
              >
                <svg v-if="todo.completed" class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
              </button>
              
              <div class="flex-1">
                <h3 :class="[
                  'font-medium text-slate-800',
                  todo.completed && 'line-through text-slate-500'
                ]">
                  {{ todo.title }}
                </h3>
                <p v-if="todo.description" :class="[
                  'text-sm text-slate-600 mt-1 whitespace-pre-line',
                  todo.completed && 'line-through text-slate-400'
                ]">
                  {{ todo.description }}
                </p>
                
                <div class="flex items-center space-x-4 mt-2">
                  <span :class="[
                    'px-2 py-1 rounded-full text-xs font-medium',
                    getPriorityColor(todo.priority)
                  ]">
                    {{ todo.priority }}
                  </span>
                  
                  <span v-if="todo.category" class="px-2 py-1 bg-slate-100 text-slate-700 rounded-full text-xs">
                    {{ todo.category }}
                  </span>
                  
                  <span class="text-xs text-slate-500">
                    {{ todo.createdAt ? `Created: ${formatDate(todo.createdAt)}` : 'No date' }}
                  </span>
                  
                  <span v-if="todo.dueDate" class="text-xs text-slate-500">
                    Due: {{ formatDate(todo.dueDate) }}
                  </span>
                </div>
              </div>
            </div>
            
            <div class="flex items-center space-x-1">
              <button 
                @click="editTodo(todo.id)"
                class="text-slate-400 hover:text-blue-500 transition-colors p-1"
                title="Edit task"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                </svg>
              </button>
              <button 
                @click="deleteTodo(todo.id)"
                class="text-slate-400 hover:text-red-500 transition-colors p-1"
                title="Delete task"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>
        </div>
        </div>
        
        <!-- Completed Tasks -->
        <div v-if="showCompletedInList && completedTodos.length > 0" class="space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-medium text-slate-600 uppercase tracking-wide">Completed Tasks</h3>
            <button 
              @click="toggleCompletedSortOrder"
              class="flex items-center gap-2 px-4 py-2 text-sm bg-green-100 text-green-700 hover:bg-green-200 rounded-lg transition-colors cursor-pointer border border-green-200"
              :title="`Sort ${completedSortOrder === 'asc' ? 'ascending' : 'descending'}`"
            >
              <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
              </svg>
              <span>{{ completedSortOrder === 'asc' ? 'Oldest First' : 'Newest First' }}</span>
            </button>
          </div>
          <div v-for="todo in completedTodos" :key="todo.id" 
               class="bg-white rounded-lg shadow-sm border border-slate-200 p-4 hover:shadow-md transition-shadow opacity-75">
            <div class="flex items-start justify-between">
              <div class="flex items-start space-x-3 flex-1">
                <button 
                  @click="toggleTodo(todo.id)"
                  :class="[
                    'mt-1 w-5 h-5 rounded border-2 flex items-center justify-center transition-colors',
                    todo.completed 
                      ? 'bg-emerald-500 border-emerald-500 text-white' 
                      : 'border-slate-300 hover:border-emerald-400'
                  ]"
                >
                  <svg v-if="todo.completed" class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                </button>
                
                <div class="flex-1">
                  <h3 :class="[
                    'font-medium text-slate-800',
                    todo.completed && 'line-through text-slate-500'
                  ]">
                    {{ todo.title }}
                  </h3>
                  <p v-if="todo.description" :class="[
                    'text-sm text-slate-600 mt-1 whitespace-pre-line',
                    todo.completed && 'line-through text-slate-400'
                  ]">
                    {{ todo.description }}
                  </p>
                  
                  <div class="flex items-center space-x-4 mt-2">
                    <span :class="[
                      'px-2 py-1 rounded-full text-xs font-medium',
                      getPriorityColor(todo.priority)
                    ]">
                      {{ todo.priority }}
                    </span>
                    
                    <span v-if="todo.category" class="px-2 py-1 bg-slate-100 text-slate-700 rounded-full text-xs">
                      {{ todo.category }}
                    </span>
                    
                    <span class="text-xs text-slate-500">
                      {{ todo.createdAt ? `Created: ${formatDate(todo.createdAt)}` : 'No date' }}
                    </span>
                    
                    <span v-if="todo.dueDate" class="text-xs text-slate-500">
                      Due: {{ formatDate(todo.dueDate) }}
                    </span>
                  </div>
                </div>
              </div>
              
              <div class="flex items-center space-x-1">
                <button 
                  @click="editTodo(todo.id)"
                  class="text-slate-400 hover:text-blue-500 transition-colors p-1"
                  title="Edit task"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button 
                  @click="deleteTodo(todo.id)"
                  class="text-slate-400 hover:text-red-500 transition-colors p-1"
                  title="Delete task"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Calendar View -->
      <div v-if="currentView === 'calendar'" class="flex flex-col lg:flex-row gap-6">
        <!-- Main Calendar -->
        <div class="flex-1 bg-white rounded-lg shadow-sm border border-slate-200 p-6">
        <!-- Debug info -->
        <div v-if="todos.length === 0" class="text-center py-8 text-slate-500">
          <p>No tasks loaded. Calendar view will show tasks when they have due dates.</p>
        </div>
        <div v-else class="text-center py-2 text-sm text-slate-500 mb-4">
          <p>Total tasks: {{ todos.length }}, Tasks with due dates: {{ todos.filter(t => t.dueDate).length }}</p>
          <div class="flex items-center justify-center gap-4 mt-2">
            <button @click="calendarUpdateKey++" class="px-3 py-1 bg-blue-500 text-white rounded text-xs">
              Force Refresh Calendar
            </button>
            <label class="flex items-center gap-2 text-xs">
              <input 
                type="checkbox" 
                v-model="showCompletedInCalendar"
                class="rounded"
              >
              Show completed tasks
            </label>
          </div>
        </div>
        <!-- Calendar Header -->
        <div class="flex items-center justify-between mb-6">
          <button 
            @click="previousMonth"
            class="p-2 hover:bg-slate-100 rounded-lg transition-colors"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          
          <h2 class="text-xl font-semibold text-slate-800">
            {{ currentMonthYear }}
          </h2>
          
          <button 
            @click="nextMonth"
            class="p-2 hover:bg-slate-100 rounded-lg transition-colors"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        </div>

        <!-- Calendar Grid -->
        <div class="grid grid-cols-7 gap-1" :key="`calendar-${calendarUpdateKey}-${currentDate.getTime()}`">
          <!-- Day Headers -->
          <div v-for="day in dayHeaders" :key="day" 
               class="p-3 text-center text-sm font-medium text-slate-600 border-b">
            {{ day }}
          </div>
          
          <!-- Calendar Days -->
          <div v-for="day in calendarDays" :key="`${day.date}-${day.isCurrentMonth}`"
               @click="selectDate(day.date)"
               :class="[
                 'p-2 min-h-[80px] border border-slate-100 cursor-pointer hover:bg-slate-50 transition-colors',
                 !day.isCurrentMonth && 'text-slate-400 bg-slate-50',
                 isToday(day.date) && 'bg-emerald-50 border-emerald-200',
                 'relative'
               ]">
            <div class="text-sm font-medium mb-1">{{ day.date.getDate() }}</div>
            
            <!-- Tasks for this day -->
            <div class="space-y-1">
              <div v-for="todo in getTasksForDate(day.date)" :key="todo.id"
                   :class="[
                     'text-xs p-1 rounded truncate',
                     getPriorityColor(todo.priority),
                     todo.completed && 'opacity-50 line-through'
                   ]">
                {{ todo.title }}
              </div>
            </div>
            
            <!-- Add task indicator -->
            <div v-if="day.isCurrentMonth" 
                 class="absolute bottom-1 right-1 w-4 h-4 bg-emerald-500 rounded-full flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity">
              <svg class="w-2 h-2 text-white" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
              </svg>
            </div>
          </div>
        </div>

        <!-- Priority Legend -->
        <div class="mt-6 flex items-center justify-center space-x-6">
          <div class="flex items-center space-x-2">
            <div class="w-3 h-3 bg-red-100 border border-red-300 rounded"></div>
            <span class="text-xs text-slate-600">High Priority</span>
          </div>
          <div class="flex items-center space-x-2">
            <div class="w-3 h-3 bg-yellow-100 border border-yellow-300 rounded"></div>
            <span class="text-xs text-slate-600">Medium Priority</span>
          </div>
          <div class="flex items-center space-x-2">
            <div class="w-3 h-3 bg-green-100 border border-green-300 rounded"></div>
            <span class="text-xs text-slate-600">Low Priority</span>
          </div>
        </div>
        </div>

        <!-- Sidebar for selected date tasks -->
        <div v-if="showSidebar" class="w-full lg:w-80 bg-white rounded-lg shadow-sm border border-slate-200 p-6">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-semibold text-slate-800">
              {{ formatDateForSidebar(selectedDate) }}
            </h3>
            <button @click="closeSidebar" class="text-slate-400 hover:text-slate-600">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div v-if="sidebarTasks.length === 0" class="text-center py-8 text-slate-500">
            <p>No tasks for this date</p>
            <button 
              @click="() => { newTask.dueDate = formatDateForInput(selectedDate); showTaskForm = true; closeSidebar(); }"
              class="mt-2 px-4 py-2 bg-emerald-500 text-white rounded-lg hover:bg-emerald-600 transition-colors"
            >
              Add Task
            </button>
          </div>

          <div v-else class="space-y-3">
            <div v-for="todo in sidebarTasks" :key="todo.id" 
                 class="group bg-slate-50 rounded-lg p-4 hover:bg-slate-100 transition-colors">
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-2 mb-2">
                    <h4 :class="[
                      'font-medium text-slate-800',
                      todo.completed && 'line-through opacity-60'
                    ]">
                      {{ todo.title }}
                    </h4>
                    <span :class="[
                      'px-2 py-1 rounded-full text-xs font-medium',
                      getPriorityColor(todo.priority)
                    ]">
                      {{ todo.priority }}
                    </span>
                  </div>
                  
                  <p v-if="todo.description" class="text-sm text-slate-600 mb-2 whitespace-pre-line">
                    {{ todo.description }}
                  </p>
                  
                  <div class="flex items-center gap-2 text-xs text-slate-500">
                    <span v-if="todo.category">{{ todo.category }}</span>
                    <span v-if="todo.createdAt">Created: {{ formatDate(todo.createdAt) }}</span>
                  </div>
                </div>
                
                <div class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button 
                    @click="toggleTodo(todo.id)"
                    :class="[
                      'p-1 rounded transition-colors',
                      todo.completed 
                        ? 'text-green-600 hover:bg-green-100' 
                        : 'text-slate-400 hover:bg-slate-200'
                    ]"
                    :title="todo.completed ? 'Mark incomplete' : 'Mark complete'"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path v-if="todo.completed" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                      <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                  </button>
                  
                  <button 
                    @click="editTodo(todo.id)"
                    class="p-1 text-slate-400 hover:text-blue-500 hover:bg-blue-100 rounded transition-colors"
                    title="Edit task"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  </button>
                  
                  <button 
                    @click="deleteTodo(todo.id)"
                    class="p-1 text-slate-400 hover:text-red-500 hover:bg-red-100 rounded transition-colors"
                    title="Delete task"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>
            
            <button 
              @click="() => { newTask.dueDate = formatDateForInput(selectedDate); showTaskForm = true; closeSidebar(); }"
              class="w-full mt-4 px-4 py-2 bg-emerald-500 text-white rounded-lg hover:bg-emerald-600 transition-colors flex items-center justify-center gap-2"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              Add Task
            </button>
          </div>
        </div>
      </div>

      <!-- Floating Action Button -->
      <button 
        @click="showTaskForm = true"
        class="fixed bottom-6 right-6 w-14 h-14 bg-emerald-500 hover:bg-emerald-600 text-white rounded-full shadow-lg hover:shadow-xl transition-all duration-200 flex items-center justify-center"
      >
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>

      <!-- Task Creation Modal -->
      <div v-if="showTaskForm" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
        <div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-semibold text-slate-800">
              {{ isEditingTask ? 'Edit Task' : 'Create New Task' }}
            </h3>
            <button @click="closeTaskForm" class="text-slate-400 hover:text-slate-600">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <form @submit.prevent="createTask" @submit="console.log('Form submitted')" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">Title</label>
              <input 
                v-model="newTask.title"
                type="text" 
                class="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500"
                placeholder="Enter task title"
              >
            </div>

            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">Description</label>
              <textarea 
                v-model="newTask.description"
                rows="3"
                class="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500"
                placeholder="Enter task description (optional)"
              ></textarea>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-slate-700 mb-1">Priority</label>
                <select 
                  v-model="newTask.priority"
                  class="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500"
                >
                  <option value="low">Low</option>
                  <option value="medium">Medium</option>
                  <option value="high">High</option>
                </select>
              </div>

              <div>
                <label class="block text-sm font-medium text-slate-700 mb-1">Category</label>
                <select 
                  v-model="newTask.category"
                  class="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500"
                >
                  <option value="">No Category</option>
                  <option value="work">Work</option>
                  <option value="personal">Personal</option>
                  <option value="shopping">Shopping</option>
                  <option value="health">Health</option>
                </select>
              </div>
            </div>

            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">Due Date</label>
              <input 
                v-model="newTask.dueDate"
                type="date"
                class="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500"
              >
            </div>

            <div class="flex space-x-3 pt-4">
              <button 
                type="button"
                @click="closeTaskForm"
                class="flex-1 px-4 py-2 border border-slate-300 text-slate-700 rounded-md hover:bg-slate-50 transition-colors"
              >
                Cancel
              </button>
              <button 
                type="submit"
                @click="console.log('Update button clicked')"
                class="flex-1 px-4 py-2 bg-emerald-500 text-white rounded-md hover:bg-emerald-600 transition-colors"
              >
                {{ isEditingTask ? 'Update Task' : 'Create Task' }}
              </button>
            </div>
          </form>
         </div>
       </div>
      </div>
    </div>
  </div>
</template>


