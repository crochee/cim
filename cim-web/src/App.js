import React, { lazy, Suspense, useContext } from "react";
import { Route, Routes, Navigate, useLocation, Outlet } from "react-router-dom";
import { AuthContext, AuthProvider } from "./context/AuthContext";

const Login = lazy(() => import("./components/Auth/Login"));
const Dashboard = lazy(() => import("./components/Dashboard"));
const UserList = lazy(() => import("./components/User/UserList"));
const UserDetail = lazy(() => import("./components/User/UserDetail"));
const GroupList = lazy(() => import("./components/Group/GroupList"));
const GroupDetail = lazy(() => import("./components/Group/GroupDetail"));
const PolicyList = lazy(() => import("./components/Policy/PolicyList"));
const PolicyDetail = lazy(() => import("./components/Policy/PolicyDetail"));

function App() {
  return (
    <AuthProvider>
      <Suspense fallback={<div>Loading...</div>}>
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route element={<PrivateRoute />}>
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/users" element={<UserList />} />
            <Route path="/users/:id" element={<UserDetail />} />
            <Route path="/groups" element={<GroupList />} />
            <Route path="/groups/:id" element={<GroupDetail />} />
            <Route path="/policies" element={<PolicyList />} />
            <Route path="/policies/:id" element={<PolicyDetail />} />
          </Route>
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Routes>
      </Suspense>
    </AuthProvider>
  );
}

const PrivateRoute = () => {
  const { user } = useContext(AuthContext);
  const location = useLocation();

  return user ? (
    <Outlet />
  ) : (
    <Navigate to={{ pathname: "/login", state: { from: location } }} replace />
  );
};

export default App;
