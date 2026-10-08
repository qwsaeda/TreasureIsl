/* ea:   0x0003E550 */
/* name: sub_3E550 */
/* size: 32 */
void __fastcall sub_3E550(void **a1)
{
  sub_3E570();
  if ( *a1 != nullptr )
    free(*a1);
}

