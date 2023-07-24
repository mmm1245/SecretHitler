const socket = new WebSocket("ws://" + location.host + "/ws");

socket.onmessage = (message) => {
  let json = JSON.parse(message.data);
  console.log(json);
  if(json.type === "SendAlert"){
    alert(json.text);
  }
  if(json.type === "PreGameUI"){
    const ui = document.getElementById("ui");
    ui.innerHTML = "";
    const roomId = document.createElement("p");
    roomId.innerText = "Room ID: " + json.room_id;
    ui.appendChild(roomId);
    let players = document.createElement("table");
    const playerCount = document.createElement("p");
    playerCount.innerText = "players: " + json.players.length + " / 10";
    ui.appendChild(playerCount);
    for(const player of json.players){
      const row = document.createElement("tr");
      const header = document.createElement("th");
      header.innerText = player;
      row.appendChild(header);
      players.appendChild(row);
    }
    ui.appendChild(players);
  }
};

document.getElementById("create_room_button").addEventListener("click", () => {
  const name = document.getElementById("name").value;
  socket.send(JSON.stringify({type:"CreateRoom",name:name}));
});

document.getElementById("join_room_button").addEventListener("click", () => {
  const name = document.getElementById("name").value;
  const room_id = document.getElementById("room_id").value;  
  socket.send(JSON.stringify({type:"JoinRoom",name:name,room_id:room_id}));
});