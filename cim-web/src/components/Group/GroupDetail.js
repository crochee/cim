import React, { useState, useEffect } from "react";
import { getGroup, updateGroup } from "../../apis";
import { useParams } from "react-router-dom";
import {
  Container,
  Typography,
  TextField,
  Button,
  List,
  ListItem,
} from "@mui/material";

const GroupDetail = () => {
  const { id } = useParams();
  const [group, setGroup] = useState(null);
  const [users, setUsers] = useState([]);

  useEffect(() => {
    const fetchGroup = async () => {
      try {
        const response = await getGroup(id);
        setGroup(response.data);
      } catch (error) {
        console.error("Failed to fetch group:", error);
      }
    };

    fetchGroup();
  }, [id]);

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      await updateGroup(id, group);
      alert("Group updated successfully");
    } catch (error) {
      console.error("Failed to update group:", error);
    }
  };

  if (!group) return <div>Loading...</div>;

  return (
    <Container>
      <Typography variant="h4" gutterBottom>
        Group Detail
      </Typography>
      <form onSubmit={handleSubmit}>
        <TextField
          label="Name"
          value={group.name}
          onChange={(e) => setGroup({ ...group, name: e.target.value })}
          fullWidth
          margin="normal"
        />
        <Button type="submit" variant="contained" color="primary">
          Save
        </Button>
      </form>
      <Typography variant="h5" gutterBottom>
        Users in Group
      </Typography>
      <List>
        {users.map((user) => (
          <ListItem key={user.id}>
            <ListItemText primary={user.name} secondary={user.email} />
          </ListItem>
        ))}
      </List>
    </Container>
  );
};

export default GroupDetail;
