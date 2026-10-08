/* ea:   0x00019B64 */
/* name: sub_19B64 */
/* size: 108 */
int __fastcall sub_19B64(_DWORD *a1, char a2, int a3)
{
  int v6; // r3
  int result; // r0
  _BYTE v8[8]; // [sp+0h] [bp-24h] BYREF
  _BYTE v9[4]; // [sp+8h] [bp-1Ch] BYREF
  int v10; // [sp+Ch] [bp-18h]

  if ( a3 == 0 )
    __debugbreak();
  v6 = sub_19DE4(*a1, a1[1], a2, v8);
  result = a1[1];
  if ( v6 == result )
  {
    v9[0] = a2;
    v10 = a3;
    return sub_19BD0(a1, v9);
  }
  return result;
}

