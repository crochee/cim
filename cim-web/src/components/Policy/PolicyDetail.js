import React, { useState, useEffect } from "react";
import { getPolicy, updatePolicy } from "../../apis";
import { useParams } from "react-router-dom";
import { Container, Typography, TextField, Button } from "@mui/material";

const PolicyDetail = () => {
  const { id } = useParams();
  const [policy, setPolicy] = useState(null);

  useEffect(() => {
    const fetchPolicy = async () => {
      try {
        const response = await getPolicy(id);
        setPolicy(response.data);
      } catch (error) {
        console.error("Failed to fetch policy:", error);
      }
    };

    fetchPolicy();
  }, [id]);

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      await updatePolicy(id, policy);
      alert("Policy updated successfully");
    } catch (error) {
      console.error("Failed to update policy:", error);
    }
  };

  if (!policy) return <div>Loading...</div>;

  return (
    <Container>
      <Typography variant="h4" gutterBottom>
        Policy Detail
      </Typography>
      <form onSubmit={handleSubmit}>
        <TextField
          label="Description"
          value={policy.desc}
          onChange={(e) => setPolicy({ ...policy, desc: e.target.value })}
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

export default PolicyDetail;
