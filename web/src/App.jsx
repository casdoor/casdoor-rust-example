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
import Users from "./Users.jsx";
import Domains from "./Domains.jsx";

const tabs = ["Profile", "Users", "Domains"];

export default function App() {
  // undefined: loading, null: signed out
  const [account, setAccount] = useState(undefined);
  const [tab, setTab] = useState("Profile");
  const [error, setError] = useState(() => new URLSearchParams(window.location.search).get("error"));

  useEffect(() => {
    // The server redirects to "/?error=..." when the sign-in fails, clean up the URL.
    if (window.location.search) {
      window.history.replaceState(null, "", "/");
    }

    api("/account")
      .then(setAccount)
      .catch(() => setAccount(null));
  }, []);

  async function signOut() {
    await api("/signout", {method: "POST"});
    setAccount(null);
    setTab("Profile");
  }

  return (
    <>
      <header>
        <h1>Casdoor Rust Example</h1>
        {account && (
          <div className="account">
            {account.avatar && <img src={account.avatar} alt="" />}
            <span>{account.displayName || account.name}</span>
            <button onClick={signOut}>Sign out</button>
          </div>
        )}
      </header>

      <main>
        {error && <p className="error">Sign-in failed: {error}</p>}

        {account === null && (
          <section className="card signin">
            <p>Sign in with Casdoor to see your profile, the users of your organization and the RBAC with domains demo.</p>
            <a className="button primary" href="/api/signin">Sign in</a>
            <a className="button" href="/api/signup">Sign up</a>
          </section>
        )}

        {account && (
          <>
            <nav>
              {tabs.map((name) => (
                <button key={name} className={tab === name ? "active" : ""} onClick={() => setTab(name)}>
                  {name}
                </button>
              ))}
            </nav>
            {tab === "Profile" && <Profile account={account} />}
            {tab === "Users" && <Users account={account} />}
            {tab === "Domains" && <Domains account={account} />}
          </>
        )}
      </main>
    </>
  );
}

function Profile({account}) {
  const rows = [
    ["ID", `${account.owner}/${account.name}`],
    ["Display name", account.displayName],
    ["Email", account.email],
    ["Admin", isAdmin(account) ? "Yes" : "No"],
    ["Created time", account.createdTime],
  ];

  return (
    <section className="card">
      <table>
        <tbody>
          {rows.map(([name, value]) => (
            <tr key={name}>
              <th>{name}</th>
              <td>{value || "-"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
