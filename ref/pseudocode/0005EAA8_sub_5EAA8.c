/* ea:   0x0005EAA8 */
/* name: sub_5EAA8 */
/* size: 76 */
int __fastcall sub_5EAA8(_DWORD *a1, int *a2)
{
  int v4; // r1
  bool v5; // cc
  int v6; // r0
  int v7; // r1
  int v8; // r0
  int v9; // r1
  int result; // r0
  int v11; // r1

  v4 = *a2;
  v5 = *a1 < v4;
  v6 = a1[2];
  if ( v5 )
    *a1 = v4;
  v7 = a2[2];
  v5 = v6 < v7;
  v8 = a1[1];
  if ( v5 )
    a1[2] = v7;
  v9 = a2[1];
  v5 = v8 <= v9;
  result = a1[3];
  if ( !v5 )
    a1[1] = v9;
  v11 = a2[3];
  if ( result > v11 )
    a1[3] = v11;
  return result;
}

