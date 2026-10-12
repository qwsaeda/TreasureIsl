/* ea:   0x0003E940 */
/* name: sub_3E940 */
/* size: 172 */
int __fastcall sub_3E940(int a1, _WORD *a2)
{
  _DWORD *v3; // r6
  int result; // r0
  int v5; // r4
  char *v6; // r4
  int v7; // [sp+0h] [bp-1Ch] BYREF

  v7 = sub_17FD8(a2);
  v3 = sub_18138(a1, (unsigned int *)&v7);
  result = v3 != *(_DWORD **)a1;
  if ( v3 != *(_DWORD **)a1 )
  {
    v5 = v3[5];
    if ( *(_DWORD *)(v5 + 4) != 0 )
    {
      (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(a1 + 16) + 12))(*(_DWORD *)(a1 + 16), *(_DWORD *)(v5 + 4));
      *(_DWORD *)(v5 + 4) = 0;
    }
    v6 = (char *)v3[5];
    if ( v6 != nullptr )
    {
      sub_58D94(v6 + 8);
      operator delete(v6);
    }
    return sub_3E9EC(a1, v3);
  }
  return result;
}

