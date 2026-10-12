/* ea:   0x0005AD7C */
/* name: sub_5AD7C */
/* size: 40 */
MMRESULT __fastcall sub_5AD7C(int a1)
{
  MMRESULT result; // r0

  result = *(unsigned __int8 *)(a1 + 68);
  if ( (_BYTE)result != 0 )
  {
    ResumeThread(*(HANDLE *)(a1 + 32));
    return waveOutRestart(*(HWAVEOUT *)a1);
  }
  return result;
}

