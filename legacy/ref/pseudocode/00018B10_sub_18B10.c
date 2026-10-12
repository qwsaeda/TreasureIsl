/* ea:   0x00018B10 */
/* name: sub_18B10 */
/* size: 116 */
int __fastcall sub_18B10(int a1)
{
  _DWORD *v2; // r4

  v2 = *(_DWORD **)(a1 + 8);
  *(_DWORD *)a1 = off_6C908;
  while ( v2 != *(_DWORD **)(a1 + 12) )
  {
    if ( *v2 != 0 )
      (*(void (__fastcall **)(_DWORD, int))(*(_DWORD *)*v2 + 4))(*v2, 1);
    ++v2;
  }
  sub_1980C(a1 + 8, *(void **)(a1 + 8), *(void **)(a1 + 12));
  sub_1984C(a1 + 8);
  return sub_19580(a1 + 4);
}

