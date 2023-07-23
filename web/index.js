const socket = new WebSocket("ws://" + location.host + "/ws");

socket.onmessage = (message) => {
  let json = JSON.parse(message.data);
  console.log(json);
  if(json.type === "SendAlert"){
    alert(json.text);
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