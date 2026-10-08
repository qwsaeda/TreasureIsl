/* ea:   0x00032F60 */
/* name: sub_32F60 */
/* size: 84 */
int __fastcall sub_32F60(int a1, _BYTE *a2)
{
  _BYTE *v4; // r0

  *(_DWORD *)a1 = 0;
  v4 = (_BYTE *)sub_34550();
  *(_DWORD *)(a1 + 4) = 0;
  *(_DWORD *)a1 = v4;
  *(_BYTE *)(a1 + 8) = *a2;
  *v4 = 0;
  *(_DWORD *)(*(_DWORD *)a1 + 4) = 0;
  *(_DWORD *)(*(_DWORD *)a1 + 8) = *(_DWORD *)a1;
  *(_DWORD *)(*(_DWORD *)a1 + 12) = *(_DWORD *)a1;
  return a1;
}

