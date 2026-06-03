---@meta

---@enum LockPickKind
LockPickKind = {
  Basic = "basic",
}

---@alias LockPickData table
LockPickData = {}

---@param kind LockPickKind
---@param data LockPickData
---@return boolean
function on_lock_pick(kind, data) end
