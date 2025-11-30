-- Install CLI Tool for Papercut
-- This script creates a symlink from /usr/local/bin/papercut to the app bundle

on run
	set appPath to "/Applications/Papercut.app/Contents/MacOS/papercut"
	set symlinkPath to "/usr/local/bin/papercut"

	-- Check if the app exists
	try
		do shell script "test -f " & quoted form of appPath
	on error
		display dialog "Error: Papercut.app not found in /Applications/" & return & return & "Please make sure Papercut.app is installed in your Applications folder before running this installer." buttons {"OK"} default button "OK" with icon stop
		return
	end try

	-- Create symlink with admin privileges
	try
		-- Ensure /usr/local/bin exists and create symlink
		do shell script "mkdir -p /usr/local/bin && ln -sf " & quoted form of appPath & " " & quoted form of symlinkPath with administrator privileges

		-- Verify the installation
		do shell script "test -L " & quoted form of symlinkPath

		display dialog "✓ CLI tool installed successfully!" & return & return & "You can now run 'papercut' from any Terminal window." & return & return & "Example:" & return & "  papercut input.py -o output.pdf" buttons {"OK"} default button "OK" with icon note

	on error errMsg
		display dialog "Installation failed:" & return & return & errMsg & return & return & "Please try running the following command manually:" & return & "sudo ln -sf /Applications/Papercut.app/Contents/MacOS/papercut /usr/local/bin/papercut" buttons {"OK"} default button "OK" with icon stop
	end try
end run
