/* ea:   0x0004E314 */
/* name: sub_4E314 */
/* size: 144 */
BOOL __fastcall sub_4E314(int a1)
{
  BOOL result; // r0
  struct _SYSTEMTIME SystemTime; // [sp+0h] [bp-1Ch] BYREF

  result = true;
  if ( *(_DWORD *)(a1 + 20) == 0 )
  {
    GetLocalTime(&SystemTime);
    srand(SystemTime.wMinute + SystemTime.wSecond + SystemTime.wMilliseconds);
    if ( rand() % 5 != 0 )
      return false;
  }
  return result;
}

