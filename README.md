# LDWP Collective Desktop App

The official desktop application for the **LDWP Collective** project.

A simple **WebView2**-based Windows app created in Visual Studio LTSC 18.11.

## Download & Run

1. Download the latest release zip
2. Extract the zip anywhere
3. Run the `.exe` file

No installation or building required.

## Prerequisites

- **Windows 10** or **Windows 11**
- **.NET 10** (`net10.0-windows`)
- **Microsoft.Web.WebView2** (version 1.0.3650.58 or compatible)

> **Note on support for Other OS:** Linux and anything else e.g. macOS/FreeBSD support is untested and unlikely due to the WebView2 requirement.
>
>  **Note for Windows users:** This app has only been built and tested on my daily driver which runs **Windows 10 IoT Enterprise LTSC 21H2**. It should still work on most other modern Windows versions if you have the dependencies needed, but that hasn't been verified.

The WebView2 Runtime is required at runtime. Most modern Windows installations already include it. If needed, it can be installed from Microsoft's official WebView2 page.

Screenshot of the application:
<img width="1202" height="832" alt="image" src="https://github.com/user-attachments/assets/58618d1b-e073-4aa3-a4c0-c4e14c20f6d1" />

## For Developers

If you're working on a new version:
- Clone the repository
- Open the solution in Visual Studio
- Build and run
