// Copyright 2022 The Casdoor Authors. All Rights Reserved.
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

import * as config from "@/config";

function getAccount() {
  return fetch(`${config.serverUrl}/api/get-account`, {
    method: 'GET',
    credentials: 'include',
  }).then(res => {
    console.log(res)
    return res.json()
  });
}

function logOut() {
  return fetch(`${config.serverUrl}/api/signout`, {
    method: 'POST',
    credentials: 'include',
  }).then((res => {
    return res.json()
  }));
}

// User management API functions
function getUserList() {
  return fetch(`${config.serverUrl}/user/list`, {
    method: 'GET',
    credentials: 'include',
  }).then(res => res.json());
}

function getUser(name) {
  return fetch(`${config.serverUrl}/user/${name}`, {
    method: 'GET',
    credentials: 'include',
  }).then(res => res.json());
}

function addUser(user) {
  return fetch(`${config.serverUrl}/user/add`, {
    method: 'POST',
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(user),
  }).then(res => res.json());
}

function updateUser(user) {
  return fetch(`${config.serverUrl}/user/update`, {
    method: 'POST',
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(user),
  }).then(res => res.json());
}

function deleteUser(user) {
  return fetch(`${config.serverUrl}/user/delete`, {
    method: 'POST',
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(user),
  }).then(res => res.json());
}

export default {
  getAccount,
  logOut,
  getUserList,
  getUser,
  addUser,
  updateUser,
  deleteUser
}
