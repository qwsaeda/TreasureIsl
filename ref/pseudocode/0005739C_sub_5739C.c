/* ea:   0x0005739C */
/* name: sub_5739C */
/* size: 72 */
int __fastcall sub_5739C(int a1)
{
  bool v2; // zf

  sub_445D8();
  v2 = off_811A0 == nullptr;
  if ( off_811A0 != nullptr )
    v2 = *((_BYTE *)off_811A0 + 32) == 0;
  if ( !v2 )
    sub_3E758(*((_DWORD *)off_811A0 + 7), *((_WORD **)off_811A0 + 9));
  return (*(int (__fastcall **)(int))(*(_DWORD *)a1 + 24))(a1);
}

