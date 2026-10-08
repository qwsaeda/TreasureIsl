/* ea:   0x00031B50 */
/* name: sub_31B50 */
/* size: 124 */
int __fastcall sub_31B50(int a1)
{
  _DWORD *v1; // r4
  _BYTE *v2; // r0
  int result; // r0
  unsigned __int8 v4[12]; // [sp+0h] [bp-Ch] BYREF

  v1 = (_DWORD *)(a1 + 32);
  ++*(_DWORD *)(a1 + 4);
  v4[0] = 0;
  do
  {
    if ( *(_BYTE *)sub_318D0(v1, v4) != 0 )
    {
      v2 = (_BYTE *)sub_318D0(v1, v4);
      --*v2;
    }
    result = v4[0]++;
  }
  while ( v4[0] < 9u );
  return result;
}

