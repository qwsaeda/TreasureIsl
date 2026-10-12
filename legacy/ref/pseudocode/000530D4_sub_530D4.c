/* ea:   0x000530D4 */
/* name: sub_530D4 */
/* size: 100 */
int __fastcall sub_530D4(char *lpBuffer, HANDLE hFile)
{
  DWORD NumberOfBytesWritten; // [sp+4h] [bp-14h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  WriteFile(hFile, lpBuffer, 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, lpBuffer + 4, 2u, &NumberOfBytesWritten, nullptr);
  return 1;
}

