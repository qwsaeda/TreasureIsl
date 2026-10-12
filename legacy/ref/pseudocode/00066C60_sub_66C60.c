/* ea:   0x00066C60 */
/* name: sub_66C60 */
/* size: 328 */
int __fastcall sub_66C60(_DWORD *a1, int a2, char *a3, int a4)
{
  int v6; // r0
  int (__fastcall *v7)(_DWORD, int, int); // r3
  int v8; // r0
  bool v9; // nf
  int (*v10)(); // r1

  if ( a3 == nullptr || *a3 != a114[0] || a4 != 56 )
    return -6;
  if ( a1 != nullptr )
  {
    v6 = a1[8];
    a1[6] = 0;
    if ( v6 == 0 )
    {
      a1[10] = 0;
      a1[8] = sub_68618;
    }
    v7 = (int (__fastcall *)(_DWORD, int, int))a1[8];
    if ( a1[9] == 0 )
      a1[9] = CMemFile::Free;
    v8 = v7(a1[10], 1, 24);
    a1[7] = v8;
    if ( v8 == 0 )
      return -4;
    *(_DWORD *)(v8 + 20) = 0;
    v9 = a2 < 0;
    if ( a2 < 0 )
      a2 = -a2;
    *(_DWORD *)(a1[7] + 12) = 0;
    if ( v9 )
      *(_DWORD *)(a1[7] + 12) = 1;
    if ( a2 >= 8 && a2 <= 15 )
    {
      *(_DWORD *)(a1[7] + 16) = a2;
      v10 = nullptr;
      if ( *(_DWORD *)(a1[7] + 12) == 0 )
        v10 = sub_68494;
      *(_DWORD *)(a1[7] + 20) = sub_6740C(a1, v10, 1 << a2);
      if ( *(_DWORD *)(a1[7] + 20) != 0 )
      {
        sub_66B9C(a1);
        return 0;
      }
      sub_66C00(a1);
      return -4;
    }
    sub_66C00(a1);
  }
  return -2;
}

