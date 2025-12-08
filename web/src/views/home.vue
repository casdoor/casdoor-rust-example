<!--Copyright 2022 The Casdoor Authors. All Rights Reserved.-->

<!--Licensed under the Apache License, Version 2.0 (the "License");-->
<!--you may not use this file except in compliance with the License.-->
<!--You may obtain a copy of the License at-->

<!--     http://www.apache.org/licenses/LICENSE-2.0 -->

<!--Unless required by applicable law or agreed to in writing, software-->
<!--distributed under the License is distributed on an "AS IS" BASIS,-->
<!--WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.-->
<!--See the License for the specific language governing permissions and-->
<!--limitations under the License.-->

<template>
  <div>
    <h2>User Profile</h2>
    <div>
      <table border="1">
        <tr>
          <th>Name</th>
          <th>Value</th>
        </tr>
        <tr>
          <td>user name</td>
          <td>{{ account.name }}</td>
        </tr>
        <tr>
          <td>user avatar</td>
          <td><img :src="account.avatar" alt="user avatar" style="width:50px"/></td>
        </tr>
        <tr>
          <td>user id</td>
          <td>{{ account.id }}</td>
        </tr>
        <tr>
          <td>user owner</td>
          <td>{{ account.owner }}</td>
        </tr>
        <tr>
          <td>user created time</td>
          <td>{{ account.createdTime }}</td>
        </tr>
      </table>
    </div>
    <br>
    <button @click="logout">Sign out</button>

    <h2>User Management</h2>
    <div>
      <table border="1" style="width: 100%; margin-bottom: 20px;">
        <thead>
          <tr>
            <th>Name</th>
            <th>DisplayName</th>
            <th>Email</th>
            <th>Phone</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in users" :key="user.name">
            <td v-if="editingUser && editingUser.name === user.name">
              <input v-model="editingUser.name" type="text" />
            </td>
            <td v-else>{{ user.name }}</td>
            <td v-if="editingUser && editingUser.name === user.name">
              <input v-model="editingUser.displayName" type="text" />
            </td>
            <td v-else>{{ user.displayName || '-' }}</td>
            <td v-if="editingUser && editingUser.name === user.name">
              <input v-model="editingUser.email" type="email" />
            </td>
            <td v-else>{{ user.email || '-' }}</td>
            <td v-if="editingUser && editingUser.name === user.name">
              <input v-model="editingUser.phone" type="tel" />
            </td>
            <td v-else>{{ user.phone || '-' }}</td>
            <td>
              <template v-if="editingUser && editingUser.name === user.name">
                <button @click="saveUser" style="margin-right: 5px;">Save</button>
                <button @click="cancelEdit">Cancel</button>
              </template>
              <template v-else>
                <button @click="editUser(user)" style="margin-right: 5px;">Edit</button>
                <button @click="deleteUser(user)">Delete</button>
              </template>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script>
import backend from '../backend/backend.js'

export default {
  name: "homePage",
  data() {
    return {
      account: {},
      users: [],
      editingUser: null,
    };
  },
  mounted() {
    let data = localStorage.getItem('user');
    console.log("data: " + data);
    this.account = JSON.parse(data);
    console.log("account: " + this.account.name);
    this.getUserList();
  },
  methods: {
    logout() {
      localStorage.removeItem('user');
      window.location.href = "/";
    },
    getUserList() {
      backend.getUserList()
        .then(users => {
          this.users = users;
        })
        .catch(error => {
          console.error('Error fetching user list:', error);
        });
    },
    editUser(user) {
      this.editingUser = JSON.parse(JSON.stringify(user));
    },
    saveUser() {
      backend.updateUser(this.editingUser)
        .then(response => {
          console.log('User updated successfully:', response);
          this.editingUser = null;
          this.getUserList();
        })
        .catch(error => {
          console.error('Error updating user:', error);
        });
    },
    cancelEdit() {
      this.editingUser = null;
    },
    deleteUser(user) {
      if (confirm(`Are you sure you want to delete user ${user.name}?`)) {
        backend.deleteUser(user)
          .then(response => {
            console.log('User deleted successfully:', response);
            this.getUserList();
          })
          .catch(error => {
            console.error('Error deleting user:', error);
          });
      }
    }
  }
}
</script>

<style scoped>
  h2 {
    margin-top: 20px;
    margin-bottom: 10px;
  }
  button {
    padding: 5px 10px;
    cursor: pointer;
  }
  input {
    padding: 3px;
  }
</style>
