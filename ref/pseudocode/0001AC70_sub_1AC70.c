/* ea:   0x0001AC70 */
/* name: sub_1AC70 */
/* size: 168 */
int __fastcall sub_1AC70(int a1, _DWORD *a2)
{
  int v4; // r1
  _DWORD *v5; // r4
  int v6; // r1
  int v7; // r2
  int v8; // r3
  int v9; // r1
  int v10; // r2
  int v11; // r3
  int result; // r0

  v4 = *(_DWORD *)(a1 + 4);
  if ( v4 == *(_DWORD *)(a1 + 8) )
    return sub_1AD18(a1);
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
    v9 = a2[5];
    v10 = a2[6];
    v11 = a2[7];
    v5[4] = a2[4];
    v5[5] = v9;
    v5[6] = v10;
    v5[7] = v11;
    v5[8] = a2[8];
  }
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(a1 + 4) = result + 36;
  return result;
}

