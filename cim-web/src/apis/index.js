import api from "./axios";

// User API
export const createUser = (user) => api.post("/users", user);
export const getUsers = () => api.get("/users");
export const getUser = (id) => api.get(`/users/${id}`);
export const updateUser = (id, user) => api.put(`/users/${id}`, user);
export const deleteUser = (id) => api.delete(`/users/${id}`);

// Group API
export const createGroup = (group) => api.post("/groups", group);
export const getGroups = () => api.get("/groups");
export const getGroup = (id) => api.get(`/groups/${id}`);
export const updateGroup = (id, group) => api.put(`/groups/${id}`, group);
export const deleteGroup = (id) => api.delete(`/groups/${id}`);

// Policy API
export const createPolicy = (policy) => api.post("/policies", policy);
export const getPolicies = () => api.get("/policies");
export const getPolicy = (id) => api.get(`/policies/${id}`);
export const updatePolicy = (id, policy) => api.put(`/policies/${id}`, policy);
export const deletePolicy = (id) => api.delete(`/policies/${id}`);

// Auth API
export const login = (username, password) => {
  return api.post("/token", {
    grant_type: "password",
    username,
    password,
  });
};

export const logout = () => {
  // Implement logout logic if needed
};
