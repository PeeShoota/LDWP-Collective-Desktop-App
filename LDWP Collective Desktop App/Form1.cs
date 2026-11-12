namespace LDWP_Collective_Desktop_App
{
    public partial class Form1 : Form
    {
        private const string WebsiteUrl = "https://ldwp.org";
        private const string WebView2RuntimeDownloadUrl = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";

        private static readonly HttpClient HttpClient = new();

        public Form1()
        {
            InitializeComponent();
            Load += Form1_Load;
        }

        private async void Form1_Load(object? sender, EventArgs e)
        {
            try
            {
                await InitializeWebViewAsync();
            }
            catch (Exception exception)
            {
                MessageBox.Show(
                    this,
                    $"The LDWP website could not be loaded. Install the Microsoft Edge WebView2 Runtime from:\n\n{WebView2RuntimeDownloadUrl}\n\n{exception.Message}",
                    "Unable to load LDWP",
                    MessageBoxButtons.OK,
                    MessageBoxIcon.Error);
            }
        }

        private async Task InitializeWebViewAsync()
        {
            try
            {
                await webView.EnsureCoreWebView2Async();
            }
            catch (Microsoft.Web.WebView2.Core.WebView2RuntimeNotFoundException)
            {
                await InstallWebView2RuntimeAsync();
                await webView.EnsureCoreWebView2Async();
            }

            webView.CoreWebView2.Navigate(WebsiteUrl);
        }

        private static async Task InstallWebView2RuntimeAsync()
        {
            string installerPath = Path.Combine(Path.GetTempPath(), $"MicrosoftEdgeWebView2Setup-{Guid.NewGuid():N}.exe");

            try
            {
                await using (FileStream installer = File.Create(installerPath))
                {
                    await using Stream download = await HttpClient.GetStreamAsync(WebView2RuntimeDownloadUrl);
                    await download.CopyToAsync(installer);
                }

                using System.Diagnostics.Process installerProcess = System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
                {
                    FileName = installerPath,
                    Arguments = "/silent /install",
                    UseShellExecute = false,
                    CreateNoWindow = true
                }) ?? throw new InvalidOperationException("The WebView2 Runtime installer could not be started.");

                await installerProcess.WaitForExitAsync();

                if (installerProcess.ExitCode != 0)
                {
                    throw new InvalidOperationException($"The WebView2 Runtime installer exited with code {installerProcess.ExitCode}.");
                }
            }
            finally
            {
                try
                {
                    if (File.Exists(installerPath))
                    {
                        File.Delete(installerPath);
                    }
                }
                catch (IOException)
                {
                }
            }
        }
    }
}
