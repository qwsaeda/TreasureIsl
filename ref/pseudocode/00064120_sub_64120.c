/* ea:   0x00064120 */
/* name: sub_64120 */
/* size: 128 */
int __fastcall sub_64120(int a1, _DWORD *a2)
{
  int v4; // r1
  _DWORD *v5; // r4
  int v6; // r1
  int v7; // r2
  int v8; // r3
  int result; // r0

  v4 = *(_DWORD *)(a1 + 4);
  if ( v4 == *(_DWORD *)(a1 + 8) )
    return sub_641E8(a1);
  v5 = *(_DWORD **)(a1 + 4);
  if ( v4 != 0 )
  {
    v6 = a2[1];
    v7 = a2[2];
    v8 = a2[3];
    *v5 = *a2;
    v5[1] = v6;
    v5[2] = v7;
    v5[3] = v8;
  }
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(a1 + 4) = result + 16;
  return result;
}

