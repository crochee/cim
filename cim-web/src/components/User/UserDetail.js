import React, { useState, useEffect } from "react";
import { getUser, updateUser } from "../../apis";
import { useParams } from "react-router-dom";
import { Container, Typography, TextField, Button } from "@mui/material";

const UserDetail = () => {
  const { id } = useParams();
  const [user, setUser] = useState(null);

  useEffect(() => {
    const fetchUser = async () => {
      try {
        const response = await getUser(id);
        setUser(response.data);
      } catch (error) {
        console.error("Failed to fetch user:", error);
      }
    };

    fetchUser();
  }, [id]);

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      await updateUser(id, user);
      alert("User updated successfully");
    } catch (error) {
      console.error("Failed to update user:", error);
    }
  };

  if (!user) return <div>Loading...</div>;

  return (
    <Container>
      <Typography variant="h4" gutterBottom>
        User Detail
      </Typography>
      <form onSubmit={handleSubmit}>
        <TextField
          label="Name"
          value={user.name}
          onChange={(e) => setUser({ ...user, name: e.target.value })}
          fullWidth
          margin="normal"
        />
        <TextField
          label="Email"
          value={user.email}
          onChange={(e) => setUser({ ...user, email: e.target.value })}
          fullWidth
          margin="normal"
        />
        <Button type="submit" variant="contained" color="primary">
          Save
        </Button>
      </form>
    </Container>
  );
};

export default UserDetail;
