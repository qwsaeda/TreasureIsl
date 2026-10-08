/* ea:   0x000300A4 */
/* name: sub_300A4 */
/* size: 32 */
void __fastcall sub_300A4(void **a1)
{
  sub_16ED8(a1);
  if ( *a1 != nullptr )
    free(*a1);
}

