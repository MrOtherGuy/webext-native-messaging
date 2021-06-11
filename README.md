# webext-native-messaging
Simple WebExtension demonstrating native messaging API

This repository contains a simple extension and source code for a native application to test WebExtension Native Messaging API.

There's three parts in the system:

#names

The extension that you can install to your browser (only Firefox probably). This sends and receives messages from the native application. This lives in the "extension" folder.

#rsio

A rust application that sends and receives messages from whatever launched it. Source code for rsio is in "application" folder.

#So how does it work?

TODO