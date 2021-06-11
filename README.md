# webext-native-messaging
Simple WebExtension demonstrating native messaging API

This repository contains a simple extension and source code for a native application to test WebExtension Native Messaging API.

There's three parts in the system:

# names

The extension that you can install to your browser (only Firefox probably). This sends and receives messages from the native application. This lives in the "extension" folder.

# rsio

A rust application that sends and receives messages from whatever launched it. Source code for rsio is in "application" folder.

# executable-folder

You would put the rsio binary here.

* rsio.conf is a configuration file that rsio.exe reads on startup
* rsio.json is the manifest file that Firefox reads to get some info about the native application it connects to.

#So how does it work?

[MDN article about native messaging is here](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging), but here's the main points:


## Connect to native application
 
The extension requests to connect to a native application. Only a background-script is allowed to do this.

```js
browser.runtime.connectNative("rsio");
```

This tells Firefox to go look for a OS specific location for a info about what this "rsio" application is. On Windows, it checks a registry key `HKEY_CURRENT_USER\SOFTWARE\Mozilla\NativeMessagingHosts\rsio`

The value of that registry key must be an absolute file path to `rsio.json`

Next, Firefox tries to read that `rsio.json` file. Among other things, the json describes where the actual binary file is. For example:

```json
{
  "name": "rsio",
  "description": "My test module",
  "type": "stdio",
  "path": "rsio.exe",
  "allowed_extensions": ["names@example.org"]
}
```

The name field might need to be the same as the json file name. The example would make Firefox **lauch** `rsio.exe` from the same path where the json file was found. Note that Firefox will launch the executable, not connect to an already running instance of the application.

The allowed_extensions field just tells that the native application wwill only receive connection from an extension of that specific ID.

If all went well, Firefox will execute the binary.

## Communication

Messages are passed using standard IO, and are composed of 32bit "header" telling the length of the message, followed by the message payload in valid JSON form. If Firefox receives standard IO that doesn't follow this format, then it will kill the application.

Extensio code doesn't need to worry about any of that. It can just send a string and let Firefox worry about any encoding and/decoding.

That 32bit "header" means that your native application cannot must do some encoding/decoding to pass anything to Firefox. It also means you should probably use some library to handle JSON formatted data.

`rsio` doesn't include any JSON library though, so there's a good chance you can crash it by sending some sort of data. Have a go and see what happens. 

## Extension

The `names` extension adds a button to toolbar that, when clicked gets the current tab url from the browser and sends it to `rsio`.

```js
port.postMessage("mirror " + tab.url)
```

The `mirror` command tells rsio to send the decoded message back to the other browser. If clicked on all went well this should log "mirror <url>" to the console of that extension. NOT the active web content console, you have to use `about:debugging` to read it.

# What command does rsio support

## ping

```
>> port.postMessage("ping")
<< "pong"
```

## mirror

```
>> port.postMessage("mirror mirror on the wall")
<< "mirror mirror on the wall"
```

## quit

Closes `rsio`, extension code receives a `ondisconnected` event.

```
>> port.postMessage("quit")
<< 
```

## config

Returns the parsed rsio.conf contents

```
>> port.postMessage("config")
<< name=MrOtherGuy
<< exec_path=C:\test\rsio_executables
```

## write

Writes text to a file. myt.txt would be created in the same folder as rsio binary.

```
>> port.postMessage("write myt.txt This should get written to myt.txt file right here")
<< wrote file: myt.txt
```

## dostuff

Tries to execute whatever you pass it to.

```
>> port.postMessage("dostuff firefox.exe https://wikipedia.org -p test")
<< Doing stuff: C:\test\rsio_executables\firefox.exe
```

Well, that assumes you have a `firefox.exe` file in `C:\test\rsio_executables\` which you probably don't. 

rsio first tries to find a file that matches the second part of the message i.e. `<path_to_rsio.exe>/firefox.exe` but if that is not a file / doesn't exist etc. Then it tries to use the `exec_path` defined in `rsio.conf` as base.

You can do something like this too:

```
>> port.postMessage("dostuff bin/firefox.exe https://wikipedia.org -p test")
<< Doing stuff: bin/putty.exe
```

That would run putty.exe if it is found in `<path_to_rsio.exe>/bin/putty.exe` If such file doesn't exist, then rsio tries to launch it from the `exec_path` again.

For the real heroes out there you could try:

```
>> port.postMessage("dostuff rsio.exe")
```

I'm not sure what will happen. Probably a whole bunch of nothing, but don't blame me if it eats your lunch or something.

There isn't currently any way to forward the output of that third-party program launched by rsio back into the extension. 

# Compiling rsio

`cd` into `application` folder and do:

```
cargo build --target=x86_64-pc-windows-gnu
```

Only tried on debian runnnig on WSL so no promises how it goes on other platforms.

After you have compiled it successfully, just copy the executable to whereever your `rsio.json` is located.

Note, you can close the old instance from Firefox using the "quit" command after which you can replace the old binary.

# What do I do with this?

I dunno, use your imagination. It's just a learning exercise.
