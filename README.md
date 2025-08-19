# What is this?

P2P file transfer client written using Iroh and egui.

There are two methods when it comes to handling files: Send or Receive.

## Send

Send generates a blob ticket using iroh, and hosts the file until stopped. 

1. Select a file to download
2. Click "send file"
3. Click "stop sending" to stop hosting the file

Note, blobs are stored in-memory. When it comes to hosting large files, please ensure you have sufficient physical RAM. Otherwise, it is recommended that you split the file into smaller blobs, generating individual tickets for the other user.

## Receive

Receive downloads the file specified in the blob ticket into the location selected.

1. Select a download location
2. Paste the blob ticket into field. Ensure no special characters appear at the end
3. Click download

A spinner will appear while the download is in progress with the number of bytes downloaded so far.

File names are encoded in base64 within the blob ticket, and will be saved accordingly on your machine.

Note, at the time of this README, that attempting to download a blob ticket that is not currently being hosted by the user will result in an empty file with the corresponding name being created at the download location.
