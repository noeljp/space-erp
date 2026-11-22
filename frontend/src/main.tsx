import { useEffect, useState, type FormEvent } from "react";
import { createRoot } from "react-dom/client";
import "./style.css";

type User = { display_name: string; role: "operator" | "supervisor" };
type Revision = { id: number; part_number: string; revision: string; description: string };
type Order = { id: number; number: string; description: string; quantity: number; status: string; total_seconds: number; part_number: string | null; revision: string | null };
type Session = { id: number; work_order_id: number; operator_name: string; started_at: string; ended_at: string | null };
type Audit = { id: number; action: string; actor: string; entity_id: number; occurred_at: string };

class ApiError extends Error {
  constructor(public status: number, message: string) { super(message); }
}

async function api<T>(path: string, token = "", method = "GET", body?: object): Promise<T> {
  const headers = new Headers();
  if (token) headers.set("Authorization", "Bearer " + token);
  if (body) headers.set("Content-Type", "application/json");
  const response = await fetch("/api/v1" + path, { method, headers, body: body && JSON.stringify(body), cache: "no-store" });
  if (!response.ok) {
    const error = await response.json().catch(() => ({}));
    throw new ApiError(response.status, error.message ?? "Requête refusée (" + response.status + ").");
  }
  return response.status === 204 ? undefined as T : response.json();
}

const date = new Intl.DateTimeFormat("fr-BE", { dateStyle: "short", timeStyle: "medium" });
const hours = new Intl.NumberFormat("fr-BE", { maximumFractionDigits: 4 });
const actions: Record<string, string> = {
  "article.revision_creation": "Création de révision article",
  "of.creation": "Création d’OF",
  "pointage.debut": "Début de pointage",
  "pointage.fin": "Fin de pointage",
};

function elapsed(seconds: number) {
  const total = Math.max(0, Math.floor(seconds));
  return [Math.floor(total / 3600), Math.floor(total / 60) % 60, total % 60]
    .map(part => String(part).padStart(2, "0")).join(":");
}

function App() {
  const [session, setSession] = useState<{ token: string; user: User }>();
  const [revisions, setRevisions] = useState<Revision[]>([]);
  const [orders, setOrders] = useState<Order[]>([]);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [audit, setAudit] = useState<Audit[]>([]);
  const [now, setNow] = useState(Date.now);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const supervisor = session?.user.role === "supervisor";
  const active = supervisor ? undefined : sessions.find(item => !item.ended_at);

  useEffect(() => {
    if (!active) return;
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, [active]);

  async function refresh(token: string, user: User) {
    const [newOrders, newSessions, newAudit, newRevisions] = await Promise.all([
      api<Order[]>("/orders", token),
      api<Session[]>("/sessions", token),
      user.role === "supervisor" ? api<Audit[]>("/audit", token) : Promise.resolve([]),
      user.role === "supervisor" ? api<Revision[]>("/item-revisions", token) : Promise.resolve([]),
    ]);
    setRevisions(newRevisions);
    setOrders(newOrders);
    setSessions(newSessions);
    setAudit(newAudit);
    setNow(Date.now());
  }

  async function run(action: () => Promise<void>) {
    setError("");
    setBusy(true);
    try {
      await action();
    } catch (problem) {
      if (session && problem instanceof ApiError && problem.status === 401) setSession(undefined);
      setError(problem instanceof Error ? problem.message : "Erreur inattendue.");
    } finally {
      setBusy(false);
    }
  }

  function login(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    void run(async () => {
      const { token } = await api<{ token: string }>("/login", "", "POST", Object.fromEntries(data));
      const user = await api<User>("/me", token);
      await refresh(token, user);
      setSession({ token, user });
    });
  }

  function mutate(path: string, body?: object, form?: HTMLFormElement) {
    if (!session) return;
    void run(async () => {
      await api(path, session.token, "POST", body);
      form?.reset();
      await refresh(session.token, session.user);
    });
  }

  function createRevision(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    mutate("/item-revisions", Object.fromEntries(new FormData(form)), form);
  }

  function create(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const data = new FormData(form);
    mutate("/orders", { number: data.get("number"), description: data.get("description"), quantity: Number(data.get("quantity")), item_revision_id: Number(data.get("item_revision_id")) }, form);
  }

  return <>
    <header><strong>SPACE ERP</strong><span>Atelier de fabrication</span></header>
    <main>
      <div className="heading">
        <div><h1>{session ? supervisor ? "Supervision atelier" : "Pointage atelier" : "Connexion à l’atelier"}</h1>
        {session && <p>{session.user.display_name}</p>}</div>
        {session && <div className="actions">
          <button disabled={busy} onClick={() => void run(() => refresh(session.token, session.user))}>Actualiser</button>
          <button disabled={busy} onClick={() => void run(async () => {
            await api("/logout", session.token, "POST");
            setSession(undefined);
          })}>Déconnexion</button>
        </div>}
      </div>
      {error && <p className="error" role="alert">{error}</p>}
      {!session ? <form onSubmit={login} className="panel login">
        <label>Identifiant<input name="username" autoComplete="username" required /></label>
        <label>Mot de passe<input name="password" type="password" autoComplete="current-password" required /></label>
        <button className="primary" disabled={busy}>Se connecter</button>
      </form> : <>
        {active && <section className="panel active" aria-label="Pointage en cours">
          <div><strong>Pointage en cours</strong><p>{orders.find(order => order.id === active.work_order_id)?.number}</p></div>
          <time>{elapsed((now - Date.parse(active.started_at)) / 1000)}</time>
          <button className="primary" disabled={busy} onClick={() => mutate("/sessions/" + active.id + "/stop")}>Terminer</button>
        </section>}
        {supervisor && <section className="panel"><h2>Articles et révisions</h2>
          <form onSubmit={createRevision} className="create">
            <label>Référence article<input name="part_number" maxLength={80} required /></label>
            <label>Révision<input name="revision" maxLength={40} required /></label>
            <label>Description de la révision<input name="description" maxLength={250} required /></label>
            <button className="primary" disabled={busy}>Créer la révision</button>
          </form>
          {revisions.length > 0 && <div className="scroll"><table>
            <thead><tr><th>Article</th><th>Révision</th><th>Description</th></tr></thead>
            <tbody>{revisions.map(item => <tr key={item.id}>
              <th scope="row">{item.part_number}</th><td>{item.revision}</td><td>{item.description}</td>
            </tr>)}</tbody>
          </table></div>}
        </section>}
        {supervisor && <section className="panel"><h2>Créer un ordre de fabrication</h2>
          <form onSubmit={create} className="create">
            <label>Numéro d’OF<input name="number" maxLength={80} required /></label>
            <label>Description<input name="description" maxLength={250} required /></label>
            <label>Quantité<input name="quantity" type="number" min="1" max="2147483647" defaultValue="1" required /></label>
            <label>Révision d’article<select name="item_revision_id" defaultValue="" required>
              <option value="" disabled>{revisions.length ? "Sélectionner une révision" : "Créez d’abord une révision"}</option>
              {revisions.map(item => <option key={item.id} value={item.id}>{item.part_number} · {item.revision}</option>)}
            </select></label>
            <button className="primary" disabled={busy || revisions.length === 0}>Créer l’OF</button>
          </form>
        </section>}
        <section className="panel"><h2>Ordres de fabrication</h2>
          {orders.length === 0 ? <p>Aucun ordre de fabrication.</p> : <div className="scroll"><table>
            <thead><tr><th>OF</th><th>Configuration</th><th>Description</th><th>Quantité</th><th>État</th><th>Heures terminées</th>{!supervisor && <th>Action</th>}</tr></thead>
            <tbody>{orders.map(order => <tr key={order.id}>
              <th scope="row">{order.number}</th><td>{order.part_number ? order.part_number + " · " + order.revision : "Aucune révision"}</td><td>{order.description}</td><td>{order.quantity}</td>
              <td>{order.status === "open" ? "Ouvert" : "Clos"}</td><td>{hours.format(order.total_seconds / 3600)} h</td>
              {!supervisor && <td><button className="primary" disabled={busy || Boolean(active) || order.status !== "open"}
                onClick={() => mutate("/orders/" + order.id + "/sessions")}>Démarrer</button></td>}
            </tr>)}</tbody>
          </table></div>}
        </section>
        <section className="panel"><h2>Historique des pointages</h2>
          {sessions.length === 0 ? <p>Aucun pointage.</p> : <div className="scroll"><table>
            <thead><tr><th>OF</th><th>Opérateur</th><th>Début</th><th>Fin</th><th>Durée</th></tr></thead>
            <tbody>{sessions.map(item => <tr key={item.id}>
              <th scope="row">{orders.find(order => order.id === item.work_order_id)?.number ?? item.work_order_id}</th>
              <td>{item.operator_name}</td><td>{date.format(new Date(item.started_at))}</td>
              <td>{item.ended_at ? date.format(new Date(item.ended_at)) : "En cours"}</td>
              <td>{item.ended_at ? elapsed((Date.parse(item.ended_at) - Date.parse(item.started_at)) / 1000) : "—"}</td>
            </tr>)}</tbody>
          </table></div>}
        </section>
        {supervisor && <section className="panel"><h2>Journal d’audit</h2>
          {audit.length === 0 ? <p>Aucun événement.</p> : <div className="scroll"><table>
            <thead><tr><th>Heure</th><th>Acteur</th><th>Action</th><th>Référence</th></tr></thead>
            <tbody>{audit.map(item => <tr key={item.id}><td>{date.format(new Date(item.occurred_at))}</td>
              <td>{item.actor}</td><td>{actions[item.action] ?? item.action}</td><td>#{item.entity_id}</td></tr>)}</tbody>
          </table></div>}
        </section>}
      </>}
    </main>
  </>;
}

createRoot(document.getElementById("root")!).render(<App />);
