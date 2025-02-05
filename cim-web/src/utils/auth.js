export const isUserAuthenticated = () => {
  const user = localStorage.getItem('user');
  return !!user;
};
