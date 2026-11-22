"""Exécute le scénario OF et pointage via le frontal. Utilise une base de démo."""
import concurrent.futures
import datetime
import json
import math
import pathlib
import sys
import urllib.error
import urllib.request
import uuid

BASE = (sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:8080").rstrip("/")
env = pathlib.Path(__file__).resolve().parents[1] / ".env"
PASSWORDS = dict(line.split("=", 1) for line in env.read_text().splitlines() if "=" in line)


def request(path, token=None, method="GET", data=None):
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    body = json.dumps(data).encode() if data is not None else None
    req = urllib.request.Request(BASE + "/api/v1" + path, body, headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=15) as response:
            return response.status, json.loads(response.read() or "null")
    except urllib.error.HTTPError as error:
        return error.code, error.read().decode()


def expect(status, path, *args):
    actual, response = request(path, *args)
    assert actual == status, (path, status, actual, response)
    return response


def login(username, key):
    return expect(200, "/login", None, "POST",
                  {"username": username, "password": PASSWORDS[key]})["token"]


with urllib.request.urlopen(BASE, timeout=15) as page:
    assert b"SPACE ERP" in page.read()
expect(200, "/health")
expect(401, "/orders")
supervisor = login("superviseur", "SUPERVISOR_PASSWORD")
operator_a = login("operateur-a", "OPERATOR_A_PASSWORD")
operator_b = login("operateur-b", "OPERATOR_B_PASSWORD")
number = f"DEMO-{uuid.uuid4().hex[:10]}"
revision = {"part_number": number, "revision": "A", "description": "Équipement fictif"}
expect(403, "/item-revisions", operator_a, "POST", revision)
revision_id = expect(201, "/item-revisions", supervisor, "POST", revision)["id"]
expect(409, "/item-revisions", supervisor, "POST", revision)
expect(201, "/item-revisions", supervisor, "POST", {**revision, "revision": "B"})
assert {"id": revision_id, **revision} in expect(200, "/item-revisions", supervisor)
order = {"number": number, "description": "Équipement fictif", "quantity": 1,
         "item_revision_id": revision_id}
expect(400, "/orders", supervisor, "POST", {**order, "item_revision_id": -1})
expect(403, "/orders", operator_a, "POST", order)
order_id = expect(201, "/orders", supervisor, "POST", order)["id"]
start_path = f"/orders/{order_id}/sessions"
with concurrent.futures.ThreadPoolExecutor(max_workers=3) as workers:
    futures = [workers.submit(request, start_path, token, "POST")
               for token in (operator_a, operator_a, operator_b)]
    result_a, duplicate_a, result_b = [future.result() for future in futures]
assert sorted([result_a[0], duplicate_a[0]]) == [201, 409], (result_a, duplicate_a)
assert result_b[0] == 201, result_b
session_a = (result_a if result_a[0] == 201 else duplicate_a)[1]["id"]
session_b = result_b[1]["id"]
expect(404, f"/sessions/{session_b}/stop", operator_a, "POST")
assert all(item["id"] != session_b for item in expect(200, "/sessions", operator_a))
running = [item for item in expect(200, "/sessions", supervisor) if item["work_order_id"] == order_id]
assert len(running) == 2 and all(item["ended_at"] is None for item in running)
expect(204, f"/sessions/{session_a}/stop", operator_a, "POST")
expect(204, f"/sessions/{session_a}/stop", operator_a, "POST")
expect(204, f"/sessions/{session_b}/stop", operator_b, "POST")
records = [item for item in expect(200, "/sessions", supervisor) if item["work_order_id"] == order_id]
parse = datetime.datetime.fromisoformat
durations = [(parse(item["ended_at"]) - parse(item["started_at"])).total_seconds() for item in records]
assert all(duration > 0 for duration in durations), durations
final_order = next(item for item in expect(200, "/orders", supervisor) if item["id"] == order_id)
assert (final_order["part_number"], final_order["revision"]) == (number, "A")
assert final_order["status"] == "open" and math.isclose(final_order["total_seconds"], sum(durations), abs_tol=1e-6)
expect(403, "/audit", operator_a)
audit = expect(200, "/audit", supervisor)
assert any(event["action"] == "article.revision_creation" and event["entity_id"] == revision_id
           for event in audit)
actions = [event["action"] for event in audit if event["payload"].get("work_order_id") == order_id
           or event["payload"].get("number") == number]
assert sorted(actions) == ["of.creation", "pointage.debut", "pointage.debut", "pointage.fin", "pointage.fin"], actions
for token in (operator_a, operator_b, supervisor):
    expect(204, "/logout", token, "POST")
    expect(401, "/me", token)
print(f"Vérifié sur {number} — deux opérateurs, conflit concurrent, total, audit et OF ouvert.")
