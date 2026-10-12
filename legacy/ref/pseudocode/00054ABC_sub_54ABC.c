/* ea:   0x00054ABC */
/* name: sub_54ABC */
/* size: 192 */
int __fastcall sub_54ABC(_DWORD *a1)
{
  int result; // r0
  int v3; // r7
  _DWORD *v4; // r8
  int v5; // r4
  _DWORD *v6; // r5
  int v7; // r2
  _DWORD *v8; // r5
  int v9; // t1
  int v10; // r0

  result = a1[266];
  v3 = 0;
  if ( result > 0 )
  {
    v4 = a1;
    do
    {
      v5 = 8 * v3;
      v6 = &a1[2 * v3];
      v9 = v6[268];
      v8 = v6 + 268;
      v7 = v9;
      if ( a1[265] != 0 )
      {
        sub_54988(a1, a1[v5 / 4u + 267], v7, *(unsigned __int8 *)(v5 + a1[257] + 6));
        v10 = a1[257];
      }
      else
      {
        sub_54988(a1, a1[v5 / 4u + 267], v7, *(unsigned __int8 *)(v5 + a1[260] + 6));
        v10 = a1[260];
      }
      sub_44828(a1, *(_DWORD *)(v5 + v10), 58, *v8 + 3);
      sub_54A5C(a1, v4[278], 118, *v8 + 3);
      result = a1[266];
      ++v3;
      ++v4;
    }
    while ( v3 < result );
  }
  return result;
}

