export interface CreateShutdownRitualStepPayload {
  title: string
}

export interface UpdateShutdownRitualStepPayload {
  id: string
  title: string
}

export interface DeleteShutdownRitualStepPayload {
  id: string
}

export interface ReorderShutdownRitualStepPayload {
  step_id: string
  prev_step_id?: string | null
  next_step_id?: string | null
}

export interface ToggleShutdownRitualProgressPayload {
  step_id: string
  date: string // YYYY-MM-DD
}

export interface UpdateShutdownRitualSettingsPayload {
  title: string | null
}


