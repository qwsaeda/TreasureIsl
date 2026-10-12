/* ea:   0x00052F18 */
/* name: sub_52F18 */
/* size: 100 */
int __fastcall sub_52F18(char *lpBuffer, HANDLE hFile)
{
  DWORD NumberOfBytesWritten; // [sp+4h] [bp-14h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  WriteFile(hFile, lpBuffer, 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(hFile, lpBuffer + 4, 4u, &NumberOfBytesWritten, nullptr);
  return 1;
}

