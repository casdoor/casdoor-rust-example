// Copyright 2026 The Casdoor Authors. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import {useEffect, useState} from "react";
import {api, isAdmin} from "./api.js";

const emptyUser = {name: "", displayName: "", password: ""};

export default function Users({account}) {
  const [users, setUsers] = useState([]);
  const [newUser, setNewUser] = useState(emptyUser);
  const [error, setError] = useState("");
  const admin = isAdmin(account);

  async function load() {
    try {
      setUsers(await api("/users"));
    } catch (e) {
      setError(e.message);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function run(action) {
    setError("");
    try {
      await action();
      await load();
    } catch (e) {
      setError(e.message);
    }
  }

  function addUser(event) {
    event.preventDefault();
    run(async () => {
      await api("/users", {method: "POST", body: newUser});
      setNewUser(emptyUser);
    });
  }

  function deleteUser(name) {
    if (window.confirm(`Delete user ${name}?`)) {
      run(() => api(`/users/${encodeURIComponent(name)}`, {method: "DELETE"}));
    }
  }

  return (
    <section className="card">
      <h2>Users of {account.owner}</h2>
      {error && <p className="error">{error}</p>}

      <table>
        <thead>
          <tr>
            <th></th>
            <th>Name</th>
            <th>Display name</th>
            <th>Email</th>
            <th>Created time</th>
            {admin && <th></th>}
          </tr>
        </thead>
        <tbody>
          {users.map((user) => (
            <tr key={user.name}>
              <td>{user.avatar && <img className="avatar" src={user.avatar} alt="" />}</td>
              <td>{user.name}</td>
              <td>{user.displayName}</td>
              <td>{user.email}</td>
              <td>{user.createdTime}</td>
              {admin && (
                <td>
                  {user.name !== account.name && (
                    <button className="danger" onClick={() => deleteUser(user.name)}>Delete</button>
                  )}
                </td>
              )}
            </tr>
          ))}
        </tbody>
      </table>

      {admin ? (
        <form onSubmit={addUser}>
          <input required placeholder="Name" value={newUser.name}
            onChange={(e) => setNewUser({...newUser, name: e.target.value})} />
          <input placeholder="Display name" value={newUser.displayName}
            onChange={(e) => setNewUser({...newUser, displayName: e.target.value})} />
          <input required type="password" placeholder="Password" value={newUser.password}
            onChange={(e) => setNewUser({...newUser, password: e.target.value})} />
          <button className="primary">Add user</button>
        </form>
      ) : (
        <p className="hint">Only the admins of the organization can add and delete users.</p>
      )}
    </section>
  );
}
