/* ea:   0x0005AD54 */
/* name: sub_5AD54 */
/* size: 40 */
DWORD __fastcall sub_5AD54(int a1)
{
  DWORD result; // r0

  result = *(unsigned __int8 *)(a1 + 68);
  if ( (_BYTE)result != 0 )
  {
    waveOutPause(*(HWAVEOUT *)a1);
    return SuspendThread(*(HANDLE *)(a1 + 32));
  }
  return result;
}

