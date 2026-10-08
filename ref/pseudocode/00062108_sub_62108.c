/* ea:   0x00062108 */
/* name: sub_62108 */
/* size: 160 */
BOOL __fastcall sub_62108(_DWORD *a1, int a2, int a3)
{
  if ( (*(unsigned __int8 (__fastcall **)(_DWORD *))(*a1 + 36))(a1) == 0 )
    __debugbreak();
  if ( a3 != 0 )
  {
    if ( a3 == 1 )
    {
      a1[5] += a2;
    }
    else if ( a3 == 2 )
    {
      a1[5] = a1[4] - a1[3] + a2;
    }
  }
  else
  {
    a1[5] = a2;
  }
  return a1[5] < (unsigned int)(*(int (__fastcall **)(_DWORD *))(*a1 + 12))(a1);
}

