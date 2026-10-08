/* ea:   0x0003D8D8 */
/* name: sub_3D8D8 */
/* size: 144 */
int __fastcall sub_3D8D8(_DWORD *a1)
{
  int v2; // r2
  int v3; // r1
  char v4; // r0
  int result; // r0

  v2 = a1[2];
  if ( v2 != 0 )
  {
    v3 = a1[15];
    if ( v3 == 0 )
      goto LABEL_8;
    if ( *(_DWORD *)(v3 + 16) != 0 || (v4 = 0, *(_DWORD *)(v3 + 20) != 0) )
      v4 = 1;
    if ( v4 != 0 )
      sub_42F94(a1[15]);
    else
LABEL_8:
      (*(void (__fastcall **)(int))(*(_DWORD *)v2 + 8))(v2);
  }
  result = a1[1];
  if ( result != 0 )
    return (*(int (__fastcall **)(_DWORD))(*(_DWORD *)a1[1] + 24))(a1[1]);
  return result;
}

