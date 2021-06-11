
let port;

function restart(){
  if(port){
    try{
      port.disconnect();
    }catch(e){
      console.log(e);
    }
  }
  setTimeout(()=>{
    port = browser.runtime.connectNative("rsio");
    port.onMessage.addListener( onMes );
    port.onDisconnect.addListener( onDis );
  },2000)
}

function onMes(mes){
  console.log("Received: ");
  console.log(mes);
}

function onDis(e){
  port.onMessage.removeListener(onMes);
  port.onDisconnect.removeListener(onDis);
  console.log("disconnected: " + e.name);
}

restart();

browser.browserAction.onClicked.addListener(() => {
  browser.tabs.query({active:true,currentWindow:true})
  .then(tabs => { 
    let tab = tabs[0];
    if(tab?.url && tab.url.startsWith("https:")){
      port.postMessage("mirror " + tab.url)
    }
  });
});
