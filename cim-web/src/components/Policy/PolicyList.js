import React, { useState, useEffect } from "react";
import { getPolicies, deletePolicy } from "../../apis";
import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Button,
} from "@mui/material";

const PolicyList = () => {
  const [policies, setPolicies] = useState([]);

  useEffect(() => {
    const fetchPolicies = async () => {
      try {
        const response = await getPolicies();
        setPolicies(response.data.data);
      } catch (error) {
        console.error("Failed to fetch policies:", error);
      }
    };

    fetchPolicies();
  }, []);

  const handleDeletePolicy = async (policyId) => {
    try {
      await deletePolicy(policyId);
      setPolicies(policies.filter((policy) => policy.id !== policyId));
    } catch (error) {
      console.error("Failed to delete policy:", error);
    }
  };

  return (
    <TableContainer component={Paper}>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell>ID</TableCell>
            <TableCell>Description</TableCell>
            <TableCell>Actions</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {policies.map((policy) => (
            <TableRow key={policy.id}>
              <TableCell>{policy.id}</TableCell>
              <TableCell>{policy.desc}</TableCell>
              <TableCell>
                <Button
                  color="primary"
                  component="a"
                  href={`/policies/${policy.id}`}
                >
                  View
                </Button>
                <Button
                  color="secondary"
                  onClick={() => handleDeletePolicy(policy.id)}
                >
                  Delete
                </Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
};

export default PolicyList;
