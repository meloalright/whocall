package pkg

import "fmt"

func FormatOutput(text string) string {
	return fmt.Sprintf("[output] %s", text)
}

func RenderText(text string) {
	formatted := FormatOutput(text)
	fmt.Println(formatted)
}
