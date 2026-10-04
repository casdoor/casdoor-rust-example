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

// RBAC with domains: a domain is a name in the "domains" field of a role or a permission,
// so domains are added and removed there. The permission's model must have a domain in
// its request, see README.md.
export default function Domains({account}) {
  const [roles, setRoles] = useState([]);
  const [permissions, setPermissions] = useState([]);
  const [error, setError] = useState("");

  async function load() {
    try {
      const [roles, permissions] = await Promise.all([api("/roles"), api("/permissions")]);
      setRoles(roles);
      setPermissions(permissions);
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

  const props = {admin: isAdmin(account), run};

  return (
    <>
      {error && <p className="error">{error}</p>}
      <DomainTable title="Roles" kind="roles" items={roles} {...props} />
      <DomainTable title="Permissions" kind="permissions" items={permissions} {...props} />
      <Enforce account={account} permissions={permissions} />
    </>
  );
}

function DomainTable({title, kind, items, admin, run}) {
  function addDomain(event, name) {
    event.preventDefault();
    const input = event.target.elements.domain;
    const domain = input.value.trim();
    if (domain) {
      run(() => api(`/${kind}/${encodeURIComponent(name)}/domains/${encodeURIComponent(domain)}`, {method: "POST"}));
      input.value = "";
    }
  }

  function deleteDomain(name, domain) {
    run(() => api(`/${kind}/${encodeURIComponent(name)}/domains/${encodeURIComponent(domain)}`, {method: "DELETE"}));
  }

  return (
    <section className="card">
      <h2>{title}</h2>
      {items.length === 0 ? (
        <p className="hint">No {kind} in this organization yet, create them in Casdoor.</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Domains</th>
            </tr>
          </thead>
          <tbody>
            {items.map((item) => (
              <tr key={item.name}>
                <td>{item.displayName || item.name}</td>
                <td>
                  {item.domains.map((domain) => (
                    <span key={domain} className="tag">
                      {domain}
                      {admin && <button title="Remove" onClick={() => deleteDomain(item.name, domain)}>×</button>}
                    </span>
                  ))}
                  {admin && (
                    <form className="inline" onSubmit={(e) => addDomain(e, item.name)}>
                      <input name="domain" placeholder="New domain" />
                      <button>Add</button>
                    </form>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function Enforce({account, permissions}) {
  const [request, setRequest] = useState({
    permissionId: "",
    user: `${account.owner}/${account.name}`,
    domain: "",
    resource: "",
    action: "",
  });
  const [result, setResult] = useState(null);

  async function enforce(event) {
    event.preventDefault();
    try {
      const allowed = await api("/enforce", {method: "POST", body: request});
      setResult(allowed ? "Allowed" : "Denied");
    } catch (e) {
      setResult(`Error: ${e.message}`);
    }
  }

  function field(name) {
    return {
      value: request[name],
      onChange: (e) => {
        setRequest({...request, [name]: e.target.value});
        setResult(null);
      },
    };
  }

  return (
    <section className="card">
      <h2>Check a right</h2>
      <p className="hint">Can the user do the action on the resource in the domain, according to the permission?</p>
      <form onSubmit={enforce}>
        <select required {...field("permissionId")}>
          <option value="">Permission</option>
          {permissions.map((p) => (
            <option key={p.name} value={`${p.owner}/${p.name}`}>{p.displayName || p.name}</option>
          ))}
        </select>
        <input required placeholder="User, e.g. my-org/alice" {...field("user")} />
        <input required placeholder="Domain" {...field("domain")} />
        <input required placeholder="Resource" {...field("resource")} />
        <input required placeholder="Action" {...field("action")} />
        <button className="primary">Check</button>
      </form>
      {result && <p className={`result ${result === "Allowed" ? "allowed" : "denied"}`}>{result}</p>}
    </section>
  );
}
